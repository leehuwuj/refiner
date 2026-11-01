//! Event-driven macOS selection tracking. AppKit callbacks never read AX;
//! all provider IPC and notification subscriptions live on one worker thread.
#![allow(unsafe_code)]

use block2::RcBlock;
use gpui_kit::{App, Global, Subscription, Task};
use objc2::{MainThreadMarker, rc::Retained, runtime::AnyObject};
use objc2_app_kit::{
    NSEvent, NSEventMask, NSEventModifierFlags, NSEventType, NSWorkspace,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::NSNotification;
use std::{
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::Duration,
};

use crate::{ax, state::AppState, windows};

enum Input {
    Down(f64, f64),
    Up {
        x: f64,
        y: f64,
        clicks: isize,
        shift: bool,
    },
    KeyDown {
        code: u16,
        shift: bool,
        command: bool,
    },
    KeyUp(u16),
    Dismiss,
}
enum Event {
    Input {
        epoch: u64,
        pid: i32,
        input: Input,
    },
    Application(u64),
    Changed {
        epoch: u64,
        pid: i32,
        focus: bool,
    },
    Result {
        epoch: u64,
        selection: Option<ax::Selection>,
    },
}
enum Command {
    Target(Option<i32>),
    Read { epoch: u64, pid: i32 },
    Stop,
}

/// A selection may be shown only after completing a gesture in its source
/// application. Delayed AX events cannot reopen the icon during a new drag.
#[derive(Default)]
struct Gate {
    down: Option<(f64, f64, i32)>,
    keys: u16,
    key_source: Option<i32>,
    ready: Option<i32>,
}
impl Gate {
    fn input(&mut self, input: Input, pid: i32) -> Option<i32> {
        match input {
            Input::Down(x, y) => {
                self.reset();
                self.down = Some((x, y, pid));
            }
            Input::Up {
                x,
                y,
                clicks,
                shift,
            } => {
                let down = self.down.take();
                let dragged =
                    down.is_some_and(|(dx, dy, _)| (dx - x).powi(2) + (dy - y).powi(2) > 16.);
                if down.is_some_and(|(_, _, source)| source == pid)
                    && (dragged || clicks >= 2 || shift)
                {
                    self.ready = Some(pid);
                    return self.ready;
                }
            }
            Input::KeyDown {
                code,
                shift,
                command,
            } => {
                self.ready = None;
                self.down = None;
                if (shift && matches!(code, 115 | 116 | 119 | 121 | 123..=126))
                    || (command && code == 0)
                {
                    if self.key_source != Some(pid) {
                        self.keys = 0;
                    }
                    self.key_source = Some(pid);
                    self.keys |= selection_key(code);
                } else {
                    self.keys = 0;
                    self.key_source = None;
                }
            }
            Input::KeyUp(code) => {
                let bit = selection_key(code);
                if self.keys & bit != 0 {
                    if self.key_source != Some(pid) {
                        self.reset();
                        return None;
                    }
                    self.keys &= !bit;
                    if self.keys == 0 {
                        self.ready = Some(pid);
                        return self.ready;
                    }
                }
            }
            Input::Dismiss => self.reset(),
        }
        None
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
    fn can_refresh(&self, pid: i32) -> bool {
        self.ready == Some(pid) && self.down.is_none() && self.keys == 0
    }
}
fn selection_key(code: u16) -> u16 {
    match code {
        0 => 1,
        115 => 2,
        116 => 4,
        119 => 8,
        121 => 16,
        123..=126 => 1 << (code - 118),
        _ => 0,
    }
}

struct Watcher {
    monitor: Option<Retained<AnyObject>>,
    local: Option<Retained<AnyObject>>,
    application: Option<Retained<AnyObject>>,
    epoch: Arc<AtomicU64>,
    since: u64,
    events: smol::channel::Sender<Event>,
    commands: mpsc::Sender<Command>,
    pending: Option<Task<()>>,
    gate: Gate,
    _settings: Subscription,
}
impl Global for Watcher {}
impl Drop for Watcher {
    fn drop(&mut self) {
        if let Some(monitor) = self.monitor.take() {
            unsafe {
                NSEvent::removeMonitor(&monitor);
            }
        }
        if let Some(local) = self.local.take() {
            unsafe {
                NSEvent::removeMonitor(&local);
            }
        }
        if let Some(observer) = self.application.take() {
            unsafe {
                NSWorkspace::sharedWorkspace()
                    .notificationCenter()
                    .removeObserver(&observer);
            }
        }
        self.epoch.fetch_add(1, Ordering::AcqRel);
        let _ = self.commands.send(Command::Stop);
    }
}

pub fn init(cx: &mut App) {
    let (events, receiver) = smol::channel::unbounded();
    let (commands, requests) = mpsc::channel();
    let epoch = Arc::new(AtomicU64::new(0));
    let worker_events = events.clone();
    let worker_epoch = epoch.clone();
    std::thread::Builder::new()
        .name("refiner-macos-selection".into())
        .spawn(move || worker(requests, worker_events, worker_epoch))
        .expect("start selection worker");
    let settings = AppState::get(cx).settings.clone();
    let subscription = cx.observe(&settings, |_, cx| sync(cx));
    cx.set_global(Watcher {
        monitor: None,
        local: None,
        application: None,
        epoch,
        since: 0,
        events,
        commands,
        pending: None,
        gate: Gate::default(),
        _settings: subscription,
    });
    sync(cx);
    cx.spawn(async move |cx| {
        while let Ok(event) = receiver.recv().await {
            cx.update(|cx| handle(event, cx));
        }
    })
    .detach();
}

fn sync(cx: &mut App) {
    let enabled = AppState::get(cx).settings.read(cx).selection_icon;
    let watcher = cx.global_mut::<Watcher>();
    if !enabled {
        watcher.since = watcher.epoch.fetch_add(1, Ordering::AcqRel) + 1;
        watcher.pending.take();
        watcher.gate.reset();
        if let Some(monitor) = watcher.monitor.take() {
            unsafe {
                NSEvent::removeMonitor(&monitor);
            }
        }
        if let Some(local) = watcher.local.take() {
            unsafe {
                NSEvent::removeMonitor(&local);
            }
        }
        if let Some(observer) = watcher.application.take() {
            unsafe {
                NSWorkspace::sharedWorkspace()
                    .notificationCenter()
                    .removeObserver(&observer);
            }
        }
        let _ = watcher.commands.send(Command::Target(None));
        windows::hide_selection_icon(cx);
        return;
    }
    if watcher.monitor.is_some() {
        return;
    }
    watcher.since = watcher.epoch.fetch_add(1, Ordering::AcqRel) + 1;
    ax::is_trusted(true);
    let tx = watcher.events.clone();
    let epoch = watcher.epoch.clone();
    let block = RcBlock::new(move |event: NonNull<NSEvent>| {
        let event = unsafe { event.as_ref() };
        let flags = event.modifierFlags();
        let p = NSEvent::mouseLocation();
        let input = match event.r#type() {
            NSEventType::LeftMouseDown => Input::Down(p.x, p.y),
            NSEventType::LeftMouseUp => Input::Up {
                x: p.x,
                y: p.y,
                clicks: event.clickCount(),
                shift: flags.contains(NSEventModifierFlags::Shift),
            },
            NSEventType::KeyDown => Input::KeyDown {
                code: event.keyCode(),
                shift: flags.contains(NSEventModifierFlags::Shift),
                command: flags.contains(NSEventModifierFlags::Command),
            },
            NSEventType::KeyUp => Input::KeyUp(event.keyCode()),
            _ => Input::Dismiss,
        };
        let id = epoch.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = tx.try_send(Event::Input {
            epoch: id,
            pid: ax::frontmost_pid().unwrap_or(0),
            input,
        });
    });
    watcher.monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
        NSEventMask::LeftMouseDown
            | NSEventMask::LeftMouseUp
            | NSEventMask::RightMouseDown
            | NSEventMask::KeyDown
            | NSEventMask::KeyUp
            | NSEventMask::ScrollWheel,
        &block,
    );
    // Global monitors exclude Refiner itself. Local interactions dismiss an
    // existing icon too, while presses on the native icon must reach its view.
    let tx = watcher.events.clone();
    let epoch = watcher.epoch.clone();
    let block = RcBlock::new(move |event: NonNull<NSEvent>| {
        let is_button = MainThreadMarker::new()
            .and_then(|mtm| unsafe { event.as_ref() }.window(mtm))
            .is_some_and(|window| crate::macos_selection_button::is_button(&window));
        if !is_button {
            let id = epoch.fetch_add(1, Ordering::AcqRel) + 1;
            let _ = tx.try_send(Event::Input {
                epoch: id,
                pid: std::process::id() as i32,
                input: Input::Dismiss,
            });
        }
        event.as_ptr()
    });
    watcher.local = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(
            NSEventMask::LeftMouseDown
                | NSEventMask::RightMouseDown
                | NSEventMask::KeyDown
                | NSEventMask::ScrollWheel,
            &block,
        )
    };
    let tx = watcher.events.clone();
    let epoch = watcher.epoch.clone();
    let block = RcBlock::new(move |_: NonNull<NSNotification>| {
        let id = epoch.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = tx.try_send(Event::Application(id));
    });
    let observer = unsafe {
        NSWorkspace::sharedWorkspace()
            .notificationCenter()
            .addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceDidActivateApplicationNotification),
                None,
                None,
                &block,
            )
    };
    watcher.application = Some(unsafe { Retained::cast_unchecked(observer) });
    let _ = watcher.commands.send(Command::Target(ax::frontmost_pid()));
}

fn handle(event: Event, cx: &mut App) {
    if !AppState::get(cx).settings.read(cx).selection_icon {
        return;
    }
    let event_epoch = match &event {
        Event::Input { epoch, .. }
        | Event::Application(epoch)
        | Event::Changed { epoch, .. }
        | Event::Result { epoch, .. } => *epoch,
    };
    if event_epoch < cx.global::<Watcher>().since {
        return;
    }
    let pid = ax::frontmost_pid().filter(|pid| *pid != std::process::id() as i32);
    match event {
        Event::Input {
            epoch,
            pid: source,
            input,
        } => {
            let refresh = cx.global_mut::<Watcher>().gate.input(input, source);
            // Process every input transition, even when a newer transition is
            // queued. The epoch check applies only to asynchronous reads.
            cx.global_mut::<Watcher>().pending.take();
            windows::hide_selection_icon(cx);
            if let Some(pid) = pid {
                let _ = cx
                    .global::<Watcher>()
                    .commands
                    .send(Command::Target(Some(pid)));
                if source == pid && refresh == Some(pid) {
                    schedule(epoch, pid, cx);
                }
            }
        }
        Event::Application(_epoch) => {
            let watcher = cx.global_mut::<Watcher>();
            watcher.pending.take();
            watcher.gate.reset();
            let _ = watcher.commands.send(Command::Target(pid));
            windows::hide_selection_icon(cx);
        }
        Event::Changed {
            epoch,
            pid: source,
            focus,
        } => {
            if pid != Some(source) {
                return;
            }
            if focus {
                let watcher = cx.global_mut::<Watcher>();
                watcher.pending.take();
                windows::hide_selection_icon(cx);
            }
            // A focus notification can arrive after mouse-up. Refresh the
            // completed gesture; presses/scrolls already reset the gate.
            if cx.global::<Watcher>().gate.can_refresh(source) {
                schedule(epoch, source, cx);
            }
        }
        Event::Result { epoch, selection } => {
            if cx.global::<Watcher>().epoch.load(Ordering::Acquire) != epoch {
                return;
            }
            match selection {
                Some(selection)
                    if pid == Some(selection.pid)
                        && cx.global::<Watcher>().gate.can_refresh(selection.pid) =>
                {
                    crate::macos_selection_button::show_at_text(selection.rect, selection.text, cx);
                }
                _ => windows::hide_selection_icon(cx),
            }
        }
    }
}

/// Opening one of Refiner's panels consumes the completed gesture. A late
/// provider notification must not reopen the icon on top of the result popup.
pub fn dismiss(cx: &mut App) {
    if cx.has_global::<Watcher>() {
        let watcher = cx.global_mut::<Watcher>();
        watcher.epoch.fetch_add(1, Ordering::AcqRel);
        watcher.pending.take();
        watcher.gate.reset();
    }
    windows::hide_selection_icon(cx);
}

fn schedule(epoch: u64, pid: i32, cx: &mut App) {
    let commands = cx.global::<Watcher>().commands.clone();
    let generation = cx.global::<Watcher>().epoch.clone();
    let task = cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_millis(80))
            .await;
        if generation.load(Ordering::Acquire) == epoch {
            let _ = commands.send(Command::Read { epoch, pid });
        }
    });
    cx.global_mut::<Watcher>().pending = Some(task);
}

fn worker(
    requests: mpsc::Receiver<Command>,
    events: smol::channel::Sender<Event>,
    epoch: Arc<AtomicU64>,
) {
    let mut target = None;
    let mut observer: Option<ax::SelectionObserver> = None;
    let mut queued = None;
    loop {
        let command = if let Some(command) = queued.take() {
            command
        } else if observer.is_some() {
            unsafe {
                core_foundation::runloop::CFRunLoopRunInMode(
                    core_foundation::runloop::kCFRunLoopDefaultMode,
                    0.04,
                    1,
                );
            }
            if let Some(observer) = &mut observer {
                observer.refresh_focus();
            }
            match requests.try_recv() {
                Ok(command) => command,
                Err(mpsc::TryRecvError::Empty) => continue,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        } else {
            match requests.recv() {
                Ok(command) => command,
                Err(_) => break,
            }
        };
        match command {
            Command::Stop => break,
            Command::Target(pid) => {
                if target != pid || (pid.is_some() && observer.is_none()) {
                    observer.take();
                    target = pid;
                    if let Some(pid) = pid {
                        let events = events.clone();
                        let epoch = epoch.clone();
                        observer = ax::SelectionObserver::new(pid, move |focus| {
                            let id = epoch.fetch_add(1, Ordering::AcqRel) + 1;
                            let _ = events.try_send(Event::Changed {
                                epoch: id,
                                pid,
                                focus,
                            });
                        });
                    }
                }
            }
            Command::Read { epoch: id, pid } => {
                if epoch.load(Ordering::Acquire) == id && target == Some(pid) {
                    let selection = ax::selection(pid);
                    debug_log!(
                        "selection: snapshot pid={pid}, bounds={:?}",
                        selection.as_ref().map(|s| s.rect)
                    );
                    if epoch.load(Ordering::Acquire) == id {
                        let _ = events.try_send(Event::Result {
                            epoch: id,
                            selection,
                        });
                    }
                }
            }
        }
        // Drain already queued input commands before waiting on the run loop.
        queued = requests.try_recv().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mouse_selection_requires_release_in_the_same_app() {
        let mut gate = Gate::default();
        gate.input(Input::Down(0., 0.), 10);
        assert!(!gate.can_refresh(10));
        assert_eq!(
            gate.input(
                Input::Up {
                    x: 100.,
                    y: 0.,
                    clicks: 1,
                    shift: false
                },
                10
            ),
            Some(10)
        );
        assert!(gate.can_refresh(10));
        gate.input(Input::Down(0., 0.), 10);
        assert!(!gate.can_refresh(10));
        assert_eq!(
            gate.input(
                Input::Up {
                    x: 100.,
                    y: 0.,
                    clicks: 1,
                    shift: false
                },
                11
            ),
            None
        );
    }
    #[test]
    fn keyboard_selection_waits_for_all_selection_keys_and_accepts_command_a() {
        let mut gate = Gate::default();
        gate.input(
            Input::KeyDown {
                code: 124,
                shift: true,
                command: false,
            },
            10,
        );
        gate.input(
            Input::KeyDown {
                code: 125,
                shift: true,
                command: false,
            },
            10,
        );
        assert_eq!(gate.input(Input::KeyUp(124), 10), None);
        assert_eq!(gate.input(Input::KeyUp(125), 10), Some(10));
        gate.input(
            Input::KeyDown {
                code: 124,
                shift: true,
                command: false,
            },
            10,
        );
        assert_eq!(gate.input(Input::KeyUp(124), 11), None);
        assert!(!gate.can_refresh(11));
        gate.input(Input::Dismiss, 10);
        assert!(!gate.can_refresh(10));
        gate.input(
            Input::KeyDown {
                code: 0,
                shift: false,
                command: true,
            },
            10,
        );
        assert_eq!(gate.input(Input::KeyUp(0), 10), Some(10));
        gate.input(
            Input::KeyDown {
                code: 1,
                shift: false,
                command: false,
            },
            10,
        );
        assert!(!gate.can_refresh(10));
    }
}
