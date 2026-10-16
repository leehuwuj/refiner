//! Floating selection icon (macOS): after the user selects text with the
//! mouse in another app, show a small button next to the cursor that opens
//! the compact popup for that text.
//!
//! A global NSEvent monitor reports mouse and key events sent to other apps.
//! A mouse-up that ends a drag, a double/triple click or a shift-click may
//! have selected text; we then read the selection through Accessibility.
//! There is deliberately no ⌘C fallback here: that would overwrite the
//! clipboard on every mouse-up. Apps that don't expose their selection simply
//! get no icon (⌘E still works there).

use gpui_kit::*;

/// Starts watching when the setting is on, and follows later changes to it.
pub fn init(cx: &mut App) {
    #[cfg(target_os = "macos")]
    macos::init(cx);
    #[cfg(not(target_os = "macos"))]
    let _ = cx;
}

/// Whether a mouse-up may have selected text.
fn may_select(down: Option<(f32, f32)>, up: (f32, f32), clicks: isize, shift: bool) -> bool {
    const DRAG_THRESHOLD: f32 = 4.;
    let dragged = down.is_some_and(|(x, y)| {
        let (dx, dy) = (up.0 - x, up.1 - y);
        dx * dx + dy * dy > DRAG_THRESHOLD * DRAG_THRESHOLD
    });
    dragged || clicks >= 2 || shift
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos {
    use std::ptr::NonNull;
    use std::time::Duration;

    use block2::RcBlock;
    use gpui_kit::*;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags, NSEventType};

    use crate::state::AppState;
    use crate::{ax, windows};

    /// Points in AppKit's bottom-left screen space.
    enum WatchEvent {
        Down(f64, f64),
        Up { x: f64, y: f64, clicks: isize, shift: bool },
        Dismiss,
    }

    struct Watcher {
        monitor: Option<Retained<AnyObject>>,
        tx: smol::channel::Sender<WatchEvent>,
        _settings: Subscription,
    }

    impl Global for Watcher {}

    pub fn init(cx: &mut App) {
        let (tx, rx) = smol::channel::unbounded::<WatchEvent>();
        let settings = AppState::get(cx).settings.clone();
        let sub = cx.observe(&settings, |_, cx| sync(cx));
        cx.set_global(Watcher {
            monitor: None,
            tx,
            _settings: sub,
        });
        sync(cx);
        cx.spawn(async move |cx| run(rx, cx).await).detach();
    }

    /// Installs or removes the event monitor to match the setting.
    fn sync(cx: &mut App) {
        let enabled = AppState::get(cx).settings.read(cx).selection_icon;
        debug_log!("selection: icon setting = {enabled}");
        let watcher = cx.global_mut::<Watcher>();
        match (enabled, watcher.monitor.is_some()) {
            (true, false) => {
                let trusted = ax::is_trusted(true);
                debug_log!("selection: accessibility trusted = {trusted}");
                watcher.monitor = install(watcher.tx.clone());
            }
            (false, true) => {
                if let Some(monitor) = watcher.monitor.take() {
                    // SAFETY: `monitor` is the object returned by
                    // addGlobalMonitorForEventsMatchingMask, removed once.
                    unsafe { NSEvent::removeMonitor(&monitor) };
                }
                windows::hide_selection_icon(cx);
            }
            _ => {}
        }
    }

    fn install(tx: smol::channel::Sender<WatchEvent>) -> Option<Retained<AnyObject>> {
        let mask = NSEventMask::LeftMouseDown
            | NSEventMask::LeftMouseUp
            | NSEventMask::RightMouseDown
            | NSEventMask::KeyDown
            | NSEventMask::ScrollWheel;
        let block = RcBlock::new(move |event: NonNull<NSEvent>| {
            // SAFETY: AppKit passes a valid NSEvent for the handler's duration.
            let event = unsafe { event.as_ref() };
            let p = NSEvent::mouseLocation();
            let msg = match event.r#type() {
                NSEventType::LeftMouseDown => WatchEvent::Down(p.x, p.y),
                NSEventType::LeftMouseUp => WatchEvent::Up {
                    x: p.x,
                    y: p.y,
                    clicks: event.clickCount(),
                    shift: event.modifierFlags().contains(NSEventModifierFlags::Shift),
                },
                _ => WatchEvent::Dismiss,
            };
            let _ = tx.try_send(msg);
        });
        let monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(mask, &block);
        if monitor.is_none() {
            eprintln!("refiner: could not install the selection monitor");
        } else {
            debug_log!("selection: global mouse monitor installed");
        }
        monitor
    }

    async fn run(rx: smol::channel::Receiver<WatchEvent>, cx: &mut AsyncApp) {
        // GPUI screen coordinates: origin at the primary display's top-left.
        let flip = |cx: &mut AsyncApp, x: f64, y: f64| {
            let height = cx.update(|cx| {
                cx.primary_display()
                    .map(|d| f32::from(d.bounds().size.height))
                    .unwrap_or(0.)
            });
            (x as f32, height - y as f32)
        };
        let mut down: Option<(f32, f32)> = None;
        while let Ok(event) = rx.recv().await {
            match event {
                WatchEvent::Down(x, y) => {
                    down = Some(flip(cx, x, y));
                    cx.update(windows::hide_selection_icon);
                }
                WatchEvent::Dismiss => {
                    down = None;
                    cx.update(windows::hide_selection_icon);
                }
                WatchEvent::Up { x, y, clicks, shift } => {
                    let up = flip(cx, x, y);
                    let start = down.take();
                    let selecting = super::may_select(start, up, clicks, shift);
                    debug_log!(
                        "selection: mouse up at {up:?} from {start:?}, clicks={clicks}, shift={shift} → {}",
                        if selecting { "check selection" } else { "ignore" }
                    );
                    if !selecting {
                        continue;
                    }
                    // Let the app finish updating its selection.
                    cx.background_executor()
                        .timer(Duration::from_millis(80))
                        .await;
                    let text = cx
                        .background_executor()
                        .spawn(async { ax::selected_text() })
                        .await;
                    // A newer click arrived while we were reading: stale.
                    if !rx.is_empty() {
                        debug_log!("selection: newer event queued, dropping result");
                        continue;
                    }
                    debug_log!("selection: AX result = {}", if text.is_some() { "text" } else { "none" });
                    if let Some(text) = text {
                        cx.update(|cx| windows::show_selection_icon(point(px(up.0), px(up.1)), text, cx));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::may_select;

    #[test]
    fn detects_selection_gestures() {
        assert!(!may_select(Some((10., 10.)), (11., 11.), 1, false));
        assert!(may_select(Some((10., 10.)), (60., 12.), 1, false));
        assert!(may_select(None, (10., 10.), 2, false));
        assert!(may_select(None, (10., 10.), 1, true));
    }
}
