//! OS integrations GPUI does not expose: grabbing the selected text from the
//! frontmost app, the global cursor position, and running as a menu-bar
//! (accessory) app on macOS.

use std::time::Duration;

/// The text selected in the frontmost app, read without touching the
/// clipboard (macOS Accessibility). `None` means "unknown": fall back to
/// [`copy_selection`]. Blocking; run it in the background.
pub fn selected_text() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        crate::ax::selected_text()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Whether reading the selection is permitted; with `prompt`, asks the OS to
/// show its permission dialog when it isn't.
pub fn selection_access(prompt: bool) -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::ax::is_trusted(prompt)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = prompt;
        true
    }
}

/// Copies the current selection in the frontmost app to the clipboard by
/// simulating Cmd+C (macOS) / Ctrl+C (Windows). Blocking; run it in the
/// background, then read the clipboard on the main thread.
pub fn copy_selection() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        const SCRIPT: &str =
            r#"tell application "System Events" to keystroke "c" using command down"#;
        let output = std::process::Command::new("osascript")
            .args(["-e", SCRIPT])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        // Give the frontmost app time to update the pasteboard.
        std::thread::sleep(Duration::from_millis(120));
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        std::thread::sleep(Duration::from_millis(150));
        windows::send_ctrl_c()?;
        std::thread::sleep(Duration::from_millis(200));
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = Duration::ZERO;
        Err("Reading the selected text is not supported on this platform".into())
    }
}

/// Explains how to grant the permission `copy_selection` needs.
pub fn show_permission_help() {
    #[cfg(target_os = "macos")]
    {
        let script = r#"display alert "Refiner needs Accessibility access" message "Refiner reads the selected text by sending ⌘C. Enable Refiner (or your terminal, when running from source) under System Settings → Privacy & Security → Accessibility, then try again." as warning"#;
        let _ = std::process::Command::new("osascript")
            .args(["-e", script])
            .spawn();
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }
}

/// Global cursor position in GPUI screen coordinates (logical pixels, origin
/// at the top-left of the primary display). `primary_height` is the primary
/// display's height, needed to flip AppKit's bottom-left origin.
pub fn cursor_position(primary_height: f32) -> Option<(f32, f32)> {
    #[cfg(target_os = "macos")]
    {
        let p = objc2_app_kit::NSEvent::mouseLocation();
        Some((p.x as f32, primary_height - p.y as f32))
    }
    #[cfg(target_os = "windows")]
    {
        let _ = primary_height;
        windows::cursor_position()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = primary_height;
        None
    }
}

/// Hides the Dock icon: Refiner lives in the menu bar.
pub fn set_accessory_app() {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod windows {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_C, VK_CONTROL,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    fn key(vk: u16, up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    pub fn send_ctrl_c() -> Result<(), String> {
        let inputs = [
            key(VK_CONTROL, false),
            key(VK_C, false),
            key(VK_C, true),
            key(VK_CONTROL, true),
        ];
        // SAFETY: `inputs` is a valid array of INPUT structs for the duration of the call.
        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            )
        };
        if sent as usize == inputs.len() {
            Ok(())
        } else {
            Err("Failed to send Ctrl+C".into())
        }
    }

    pub fn cursor_position() -> Option<(f32, f32)> {
        let mut p = POINT { x: 0, y: 0 };
        // SAFETY: `p` is a valid, writable POINT.
        if unsafe { GetCursorPos(&mut p) } == 0 {
            return None;
        }
        // SAFETY: no preconditions.
        let scale = unsafe { GetDpiForSystem() } as f32 / 96.0;
        let scale = if scale > 0.0 { scale } else { 1.0 };
        Some((p.x as f32 / scale, p.y as f32 / scale))
    }
}
