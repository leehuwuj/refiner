//! Observe selection gesture boundaries. Hooks never swallow or inject input.
#![allow(unsafe_code)]

use crate::windows_selection_watch::{Command, IconEvent};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, mpsc},
};
use windows_sys::Win32::{
    Foundation::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Completed {
    pub foreground: usize,
    pub mouse_end: Option<(i32, i32)>,
}

#[derive(Default, Debug)]
pub(super) struct InputState {
    pub epoch: u64,
    mouse_down: bool,
    mouse_source: usize,
    shift: u8,
    control: u8,
    selecting_keys: u16,
    pub completed: Option<Completed>,
}

impl InputState {
    fn invalidate(&mut self) {
        self.epoch += 1;
        self.completed = None;
    }
    fn mouse_down(&mut self, foreground: usize) {
        self.invalidate();
        self.mouse_down = true;
        self.mouse_source = foreground;
        self.selecting_keys = 0;
    }
    fn mouse_up(&mut self, end: (i32, i32)) {
        if self.mouse_down {
            self.mouse_down = false;
            self.epoch += 1;
            self.completed = Some(Completed {
                foreground: self.mouse_source,
                mouse_end: Some(end),
            });
        }
    }
    fn key_down(&mut self, key: u32) {
        self.invalidate();
        match key {
            0x10 | 0xa0 => self.shift |= 1,
            0xa1 => self.shift |= 2,
            0x11 | 0xa2 => self.control |= 1,
            0xa3 => self.control |= 2,
            _ => {}
        }
        if (self.shift != 0 && matches!(key, 0x21..=0x28)) || (self.control != 0 && key == 0x41) {
            self.selecting_keys |= selection_key(key);
        }
    }
    fn key_up(&mut self, key: u32, foreground: usize) {
        match key {
            0x10 | 0xa0 => self.shift &= !1,
            0xa1 => self.shift &= !2,
            0x11 | 0xa2 => self.control &= !1,
            0xa3 => self.control &= !2,
            _ => {}
        }
        let selected = self.selecting_keys & selection_key(key) != 0;
        self.selecting_keys &= !selection_key(key);
        if selected && self.selecting_keys == 0 {
            self.epoch += 1;
            self.completed = Some(Completed {
                foreground,
                mouse_end: None,
            });
        }
    }
    pub fn completion(&self) -> Option<Completed> {
        if self.mouse_down || self.selecting_keys != 0 {
            None
        } else {
            self.completed
        }
    }

    fn dismiss(&mut self) {
        self.invalidate();
        self.mouse_down = false;
        self.selecting_keys = 0;
    }
}

fn selection_key(key: u32) -> u16 {
    match key {
        0x21..=0x28 => 1 << (key - 0x21),
        0x41 => 1 << 8,
        _ => 0,
    }
}

#[derive(Debug, PartialEq)]
enum MouseAction {
    Start(usize),
    Finish,
    Dismiss,
    Ignore,
}

fn mouse_action(message: u32, owned: bool, icon: bool, root: usize) -> MouseAction {
    match message {
        WM_LBUTTONUP => MouseAction::Finish,
        WM_LBUTTONDOWN if icon => MouseAction::Ignore,
        WM_LBUTTONDOWN if !owned => MouseAction::Start(root),
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            MouseAction::Dismiss
        }
        _ => MouseAction::Ignore,
    }
}

pub(super) type Gate = Arc<Mutex<InputState>>;

struct Context {
    gate: Gate,
    commands: mpsc::SyncSender<Command>,
    events: smol::channel::Sender<IconEvent>,
}
thread_local! { static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) }; }

pub(super) struct InputHooks {
    mouse: HHOOK,
    keyboard: HHOOK,
}
impl InputHooks {
    pub fn install(
        gate: Gate,
        commands: mpsc::SyncSender<Command>,
        events: smol::channel::Sender<IconEvent>,
    ) -> Option<Self> {
        unsafe {
            let module = GetModuleHandleW(std::ptr::null());
            let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_event), module, 0);
            let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_event), module, 0);
            if mouse.is_null() || keyboard.is_null() {
                if !mouse.is_null() {
                    UnhookWindowsHookEx(mouse);
                }
                if !keyboard.is_null() {
                    UnhookWindowsHookEx(keyboard);
                }
                eprintln!(
                    "refiner: cannot observe selection gesture completion: {}",
                    std::io::Error::last_os_error()
                );
                return None;
            }
            CONTEXT.with(|context| {
                *context.borrow_mut() = Some(Context {
                    gate,
                    commands,
                    events,
                })
            });
            debug_log!("selection: native mouse/key release observers installed");
            Some(Self { mouse, keyboard })
        }
    }
}
impl Drop for InputHooks {
    fn drop(&mut self) {
        unsafe {
            UnhookWindowsHookEx(self.mouse);
            UnhookWindowsHookEx(self.keyboard);
        }
        CONTEXT.with(|context| context.borrow_mut().take());
    }
}

fn update(f: impl FnOnce(&mut InputState)) {
    CONTEXT.with(|context| {
        if let Some(context) = context.borrow().as_ref() {
            let epoch = {
                let mut state = context.gate.lock().unwrap();
                let old = state.epoch;
                f(&mut state);
                if old == state.epoch {
                    return;
                }
                state.epoch
            };
            // Clear old UI immediately; a release triggers one fresh UIA read.
            let _ = context.events.force_send((epoch, None));
            let _ = context.commands.try_send(Command::Refresh);
        }
    });
}

unsafe extern "system" fn mouse_event(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    unsafe {
        if code == HC_ACTION as i32
            && matches!(
                message as u32,
                WM_LBUTTONDOWN
                    | WM_LBUTTONUP
                    | WM_RBUTTONDOWN
                    | WM_MBUTTONDOWN
                    | WM_MOUSEWHEEL
                    | WM_MOUSEHWHEEL
            )
        {
            let event = &*(data as *const MSLLHOOKSTRUCT);
            let target = WindowFromPoint(event.pt);
            let mut pid = 0;
            GetWindowThreadProcessId(target, &mut pid);
            let action = mouse_action(
                message as u32,
                pid == std::process::id(),
                crate::windows_selection_button::is_button(target),
                GetAncestor(target, GA_ROOT) as usize,
            );
            match action {
                MouseAction::Start(root) => update(|state| state.mouse_down(root)),
                MouseAction::Finish => update(|state| state.mouse_up((event.pt.x, event.pt.y))),
                MouseAction::Dismiss => update(|state| state.dismiss()),
                MouseAction::Ignore => {}
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, message, data)
    }
}

unsafe extern "system" fn key_event(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    unsafe {
        if code == HC_ACTION as i32 {
            let event = &*(data as *const KBDLLHOOKSTRUCT);
            match message as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => update(|state| state.key_down(event.vkCode)),
                WM_KEYUP | WM_SYSKEYUP => {
                    update(|state| state.key_up(event.vkCode, GetForegroundWindow() as usize))
                }
                _ => {}
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, message, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn releasing_over_refiner_finishes_drag_but_clicking_icon_preserves_selection() {
        let mut state = InputState::default();
        state.mouse_down(42);
        assert_eq!(
            mouse_action(WM_LBUTTONUP, true, false, 99),
            MouseAction::Finish
        );
        state.mouse_up((10, 10));
        assert!(state.completion().is_some());
        assert_eq!(
            mouse_action(WM_LBUTTONDOWN, true, true, 99),
            MouseAction::Ignore
        );
        assert_eq!(
            mouse_action(WM_LBUTTONDOWN, true, false, 99),
            MouseAction::Dismiss
        );
        state.dismiss();
        state.mouse_up((10, 10));
        assert_eq!(state.completion(), None);
    }
    #[test]
    fn all_selection_keys_must_be_released_and_both_shift_keys_are_tracked() {
        let mut state = InputState::default();
        state.key_down(0xa0);
        state.key_down(0xa1);
        state.key_up(0xa0, 42); // Right Shift remains held.
        state.key_down(0x27);
        state.key_down(0x26);
        state.key_up(0x26, 42);
        assert_eq!(state.completion(), None);
        state.key_up(0x27, 42);
        assert!(state.completion().is_some());
    }
    #[test]
    fn drag_is_hidden_until_release_and_resets_on_next_gesture() {
        let mut state = InputState::default();
        assert_eq!(state.completion(), None);
        state.mouse_down(42);
        for _ in 0..100 {
            assert_eq!(state.completion(), None);
        }
        state.mouse_up((100, 200));
        assert_eq!(
            state.completion(),
            Some(Completed {
                foreground: 42,
                mouse_end: Some((100, 200))
            })
        );
        state.mouse_down(42);
        assert_eq!(state.completion(), None);
    }
    #[test]
    fn completion_uses_the_release_point_and_invalidates_old_results() {
        let mut state = InputState::default();
        state.mouse_down(42);
        state.mouse_up((-150, 300));
        let finished_epoch = state.epoch;
        assert_eq!(state.completion().unwrap().mouse_end, Some((-150, 300)));
        state.mouse_down(99);
        assert_ne!(state.epoch, finished_epoch);
        assert_eq!(state.completion(), None);
        state.mouse_up((20, 50));
        assert_eq!(
            state.completion(),
            Some(Completed {
                foreground: 99,
                mouse_end: Some((20, 50))
            })
        );
    }
    #[test]
    fn keyboard_selection_waits_for_key_release() {
        let mut state = InputState::default();
        state.key_down(0x10); // Shift
        state.key_down(0x27); // Right
        state.key_down(0x27); // auto-repeat
        assert_eq!(state.completion(), None);
        state.key_up(0x27, 42);
        assert_eq!(
            state.completion(),
            Some(Completed {
                foreground: 42,
                mouse_end: None
            })
        );
        state.key_down(0x42); // typing clears it
        assert_eq!(state.completion(), None);
        state.key_down(0x11); // Ctrl+A
        state.key_down(0x41);
        assert_eq!(state.completion(), None);
        state.key_up(0x41, 42);
        assert!(state.completion().is_some());
    }
}
