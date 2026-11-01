//! Event-driven Windows selection icon. No clipboard, key injection or polling.

#![allow(unsafe_code)]

use gpui_kit::{App, Global, Subscription};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Accessibility::*;
use windows::core::{ComObjectInner, Result, implement};

use crate::state::AppState;
use crate::windows as app_windows;
use crate::windows_selection::{ComApartment, Selection, current_selection};
use crate::windows_selection_input::{Gate, InputHooks, InputState};

pub(super) enum Command {
    Refresh,
    Stop,
}

pub(super) type IconEvent = (u64, Option<Selection>);

#[implement(IUIAutomationEventHandler, IUIAutomationFocusChangedEventHandler)]
struct Handler {
    commands: mpsc::SyncSender<Command>,
}

impl IUIAutomationEventHandler_Impl for Handler_Impl {
    fn HandleAutomationEvent(
        &self,
        _: Option<&IUIAutomationElement>,
        _: UIA_EVENT_ID,
    ) -> Result<()> {
        let _ = self.commands.try_send(Command::Refresh);
        Ok(())
    }
}

impl IUIAutomationFocusChangedEventHandler_Impl for Handler_Impl {
    fn HandleFocusChangedEvent(&self, _: Option<&IUIAutomationElement>) -> Result<()> {
        let _ = self.commands.try_send(Command::Refresh);
        Ok(())
    }
}

struct Monitor {
    commands: mpsc::SyncSender<Command>,
    active: Arc<AtomicBool>,
    gate: Gate,
    _input: Option<InputHooks>,
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
        // If full, a pending Refresh already wakes the worker, which checks
        // active before reading. Never block the UI thread during teardown.
        let _ = self.commands.try_send(Command::Stop);
    }
}

struct Watcher {
    monitor: Option<Monitor>,
    _settings: Subscription,
}
impl Global for Watcher {}

pub fn init(cx: &mut App) {
    let settings = AppState::get(cx).settings.clone();
    let subscription = cx.observe(&settings, |_, cx| sync(cx));
    cx.set_global(Watcher {
        monitor: None,
        _settings: subscription,
    });
    sync(cx);
}

fn sync(cx: &mut App) {
    let enabled = AppState::get(cx).settings.read(cx).selection_icon;
    if !enabled {
        cx.global_mut::<Watcher>().monitor.take();
        app_windows::hide_selection_icon(cx);
    } else if cx.global::<Watcher>().monitor.is_none() {
        let (monitor, events) = start();
        let active = monitor.active.clone();
        let gate = monitor.gate.clone();
        cx.global_mut::<Watcher>().monitor = Some(monitor);
        cx.spawn(async move |cx| {
            while let Ok((epoch, selection)) = events.recv().await {
                if !active.load(Ordering::Acquire) {
                    break;
                }
                cx.update(|cx| {
                    if !AppState::get(cx).settings.read(cx).selection_icon {
                        return;
                    }
                    let state = gate.lock().unwrap();
                    if state.epoch != epoch { return; }
                    let completion = state.completion();
                    drop(state);
                    match selection {
                        Some(selection)
                            if completion.is_some_and(|done| done.foreground == selection.foreground) && unsafe {
                                windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow()
                            } as usize
                                == selection.foreground =>
                        {
                            if let Some((x, y)) = selection.at {
                                crate::windows_selection_button::show_at_text((x, y), selection.text, cx);
                            } else {
                                app_windows::hide_selection_icon(cx);
                            }
                        }
                        _ => app_windows::hide_selection_icon(cx),
                    }
                });
                // Let native messages, paint and SVG loading run between UIA
                // updates, even when a drag keeps producing selection events.
                smol::future::yield_now().await;
            }
        })
        .detach();
    }
}

fn start() -> (Monitor, smol::channel::Receiver<IconEvent>) {
    let (commands, requests) = mpsc::sync_channel(1);
    // The icon represents the current selection, not a history of drag steps.
    let (events, receiver) = smol::channel::bounded(1);
    let active = Arc::new(AtomicBool::new(true));
    let gate = Arc::new(std::sync::Mutex::new(InputState::default()));
    let input = InputHooks::install(gate.clone(), commands.clone(), events.clone());
    let monitor = Monitor {
        commands: commands.clone(),
        active: active.clone(),
        gate: gate.clone(),
        _input: input,
    };
    let _ = std::thread::Builder::new()
        .name("refiner-selection-events".into())
        .spawn(move || {
            if let Err(err) = listen(commands, requests, events, active, gate) {
                eprintln!("refiner: selection event listener unavailable: {err}");
            }
        });
    (monitor, receiver)
}

fn listen(
    commands: mpsc::SyncSender<Command>,
    requests: mpsc::Receiver<Command>,
    events: smol::channel::Sender<IconEvent>,
    active: Arc<AtomicBool>,
    gate: Gate,
) -> Result<()> {
    // Initialize, register and unregister on this MTA thread. Callbacks only
    // enqueue a signal; provider reads happen here, outside callback stacks.
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let _apartment = ComApartment;
        let automation: IUIAutomation2 =
            CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)?;
        automation.SetConnectionTimeout(500)?;
        automation.SetTransactionTimeout(500)?;
        let _ = commands.try_send(Command::Refresh);
        let handler = Handler { commands }.into_object();
        let selection_handler: IUIAutomationEventHandler = handler.to_interface();
        let focus_handler: IUIAutomationFocusChangedEventHandler = handler.to_interface();
        automation.AddFocusChangedEventHandler(None, &focus_handler)?;
        debug_log!("selection: UIA focus event handler installed");
        let mut previous = None;
        let mut last_focus: Option<IUIAutomationElement> = None;
        let mut source: Option<IUIAutomationElement> = None;
        while let Ok(command) = requests.recv() {
            if matches!(command, Command::Stop) || !active.load(Ordering::Acquire) {
                break;
            }
            let focused = automation.GetFocusedElement().ok();
            let same_focus = match (&last_focus, &focused) {
                (Some(old), Some(new)) => automation
                    .CompareElements(old, new)
                    .is_ok_and(|same| same.as_bool()),
                (None, None) => true,
                _ => false,
            };
            if !same_focus {
                if let Some(old) = source.take() {
                    let _ = automation.RemoveAutomationEventHandler(
                        UIA_Text_TextSelectionChangedEventId,
                        &old,
                        &selection_handler,
                    );
                }
                source = focused
                    .as_ref()
                    .and_then(|focused| text_source(&automation, focused));
                if let Some(element) = &source
                    && let Err(err) = automation.AddAutomationEventHandler(
                        UIA_Text_TextSelectionChangedEventId,
                        element,
                        TreeScope_Element,
                        None,
                        &selection_handler,
                    )
                {
                    debug_log!("selection: provider does not support event registration: {err}");
                    source = None;
                }
                last_focus = focused;
            }
            let (epoch, completion) = {
                let state = gate.lock().unwrap();
                (state.epoch, state.completion())
            };
            let selection = completion.and_then(|done| {
                let selection = current_selection(&automation)?;
                if selection.foreground != done.foreground {
                    return None;
                }
                Some(selection)
            });
            // A new press may arrive during a provider call. Never publish a
            // selection from the previous gesture into the new one.
            if gate.lock().unwrap().epoch != epoch {
                continue;
            }
            let event = (epoch, selection);
            if previous.as_ref() != Some(&event) {
                previous = Some(event.clone());
                let _ = events.force_send(event);
            }
        }
        let _ = automation.RemoveFocusChangedEventHandler(&focus_handler);
        if let Some(element) = source {
            let _ = automation.RemoveAutomationEventHandler(
                UIA_Text_TextSelectionChangedEventId,
                &element,
                &selection_handler,
            );
        }
        debug_log!("selection: UIA event handlers removed");
    }
    Ok(())
}

fn text_source(
    automation: &IUIAutomation2,
    focused: &IUIAutomationElement,
) -> Option<IUIAutomationElement> {
    unsafe {
        if focused.CurrentProcessId().ok()? as u32 == std::process::id() {
            return None;
        }
        let walker = automation.ControlViewWalker().ok()?;
        let mut element = focused.clone();
        for _ in 0..32 {
            if element.CurrentIsPassword().ok()?.as_bool() {
                return None;
            }
            if element
                .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
                .is_ok()
            {
                return Some(element);
            }
            element = walker.GetParentElement(&element).ok()?;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use windows::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, MSG, PM_REMOVE, PeekMessageW,
        SendMessageW, WS_POPUP,
    };

    #[test]
    fn native_edit_emits_selection_change_event() {
        let _serial = crate::windows_selection::NATIVE_UIA_TEST.lock().unwrap();
        unsafe {
            let hwnd = CreateWindowExW(
                0,
                windows::core::w!("EDIT").as_ptr(),
                windows::core::w!("Hello 世界").as_ptr(),
                WS_POPUP,
                0,
                0,
                200,
                50,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            assert!(!hwnd.is_null());
            let (result_tx, result_rx) = mpsc::sync_channel(1);
            let native = hwnd as usize;
            std::thread::spawn(move || {
                let result = (|| -> Result<()> {
                    CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
                    let _apartment = ComApartment;
                    let automation: IUIAutomation2 =
                        CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)?;
                    let element = automation.ElementFromHandle(HWND(native as _))?;
                    let (commands, events) = mpsc::sync_channel(1);
                    let handler: IUIAutomationEventHandler = Handler { commands }.into();
                    automation.AddAutomationEventHandler(
                        UIA_Text_TextSelectionChangedEventId,
                        &element,
                        TreeScope_Element,
                        None,
                        &handler,
                    )?;
                    SendMessageW(native as _, 0x00B1, 6, 8); // EM_SETSEL
                    let received = events.recv_timeout(Duration::from_secs(3));
                    automation.RemoveAutomationEventHandler(
                        UIA_Text_TextSelectionChangedEventId,
                        &element,
                        &handler,
                    )?;
                    assert!(
                        matches!(received, Ok(Command::Refresh)),
                        "native selection event not received"
                    );
                    Ok(())
                })();
                let _ = result_tx.send(result);
            });
            let started = Instant::now();
            let result = loop {
                if let Ok(result) = result_rx.try_recv() {
                    break result;
                }
                assert!(
                    started.elapsed() < Duration::from_secs(10),
                    "event fixture timed out"
                );
                let mut message: MSG = std::mem::zeroed();
                while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    DispatchMessageW(&message);
                }
                std::thread::sleep(Duration::from_millis(5));
            };
            DestroyWindow(hwnd);
            result.expect("native UIA event registration");
        }
    }
}
