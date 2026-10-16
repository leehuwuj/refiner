//! Global shortcut (⌘E on macOS, Ctrl+E elsewhere): grabs the selected text
//! in the frontmost app and processes it in the main window or the popup.

use std::time::{Duration, Instant};

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui_kit::*;

use crate::platform;
use crate::settings::ShortcutWindow;
use crate::state::AppState;
use crate::windows;

const DEBOUNCE: Duration = Duration::from_millis(300);

/// Keeps the registration alive for the lifetime of the app.
struct Hotkeys {
    _manager: GlobalHotKeyManager,
}

impl Global for Hotkeys {}

pub fn init(cx: &mut App) {
    let modifiers = if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL
    };
    let hotkey = HotKey::new(Some(modifiers), Code::KeyE);
    let manager = match GlobalHotKeyManager::new() {
        Ok(m) => m,
        Err(err) => {
            eprintln!("refiner: global shortcuts unavailable: {err}");
            return;
        }
    };
    if let Err(err) = manager.register(hotkey) {
        eprintln!("refiner: failed to register the global shortcut: {err}");
        return;
    }
    cx.set_global(Hotkeys { _manager: manager });
    // Ask for Accessibility access up front so the first ⌘E just works.
    platform::selection_access(true);

    let id = hotkey.id();
    let (tx, rx) = smol::channel::unbounded::<()>();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.id == id && event.state == HotKeyState::Pressed {
            let _ = tx.try_send(());
        }
    }));

    cx.spawn(async move |cx| {
        let mut last: Option<Instant> = None;
        while rx.recv().await.is_ok() {
            if last.is_some_and(|t| t.elapsed() < DEBOUNCE) {
                continue;
            }
            last = Some(Instant::now());
            debug_log!("hotkey: pressed");
            // Accessibility first: no clipboard side effects. Apps that don't
            // expose their selection (terminals, some Electron/Java apps)
            // fall back to simulating ⌘C.
            let grabbed = cx
                .background_executor()
                .spawn(async {
                    if let Some(text) = platform::selected_text() {
                        return Ok(Some(text));
                    }
                    debug_log!("hotkey: no AX selection, falling back to ⌘C");
                    platform::copy_selection().map(|()| None)
                })
                .await;
            cx.update(|cx| match grabbed {
                Ok(Some(text)) => process_text(text, cx),
                Ok(None) => process_clipboard(cx),
                Err(err) => {
                    eprintln!("refiner: could not copy the selection: {err}");
                    platform::show_permission_help();
                }
            });
        }
    })
    .detach();
}

fn process_clipboard(cx: &mut App) {
    let text = cx
        .read_from_clipboard()
        .and_then(|item| item.text())
        .map(|t| t.trim().to_string())
        .unwrap_or_default();
    process_text(text, cx);
}

/// Sends `text` to the window chosen in settings.
pub fn process_text(text: String, cx: &mut App) {
    if text.is_empty() {
        return;
    }
    let state = AppState::get(cx).clone();
    match state.settings.read(cx).shortcut_window {
        ShortcutWindow::Popup => {
            state.popup.update(cx, |p, cx| p.run(text, cx));
            windows::show_popup(cx);
        }
        ShortcutWindow::Main => {
            state.main.update(cx, |m, cx| {
                m.set_input(text, cx);
                m.run(cx);
            });
            windows::show_main(cx);
        }
    }
}
