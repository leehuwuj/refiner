//! OS integrations GPUI does not expose: grabbing the selected text from the
//! frontmost app, the global cursor position, and running as a menu-bar
//! (accessory) app on macOS.

#[cfg(target_os = "macos")]
use std::time::Duration;

/// The text selected in the frontmost app, read without touching the
/// clipboard (macOS Accessibility / Windows UI Automation). `None` means
/// no readable selection. Blocking; run it in the background.
pub fn selected_text() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        crate::ax::selected_text()
    }
    #[cfg(target_os = "windows")]
    {
        crate::windows_selection::selected_text()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
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

/// macOS-only compatibility path. Windows never synthesizes copy shortcuts.
#[cfg(target_os = "macos")]
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
}

/// Explains how to grant the permission `copy_selection` needs.
#[cfg(target_os = "macos")]
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
pub fn cursor_position(primary_height: f32) -> Option<(f32, f32, Option<u64>)> {
    #[cfg(target_os = "macos")]
    {
        let p = objc2_app_kit::NSEvent::mouseLocation();
        Some((p.x as f32, primary_height - p.y as f32, None))
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
    use windows_sys::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromPoint};
    use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    pub fn cursor_position() -> Option<(f32, f32, Option<u64>)> {
        let mut p = POINT { x: 0, y: 0 };
        // SAFETY: `p` is a valid, writable POINT.
        if unsafe { GetCursorPos(&mut p) } == 0 {
            return None;
        }
        // Match GPUI's per-monitor coordinate conversion, including mixed-DPI
        // displays. System DPI is only correct on the primary monitor.
        let monitor = unsafe { MonitorFromPoint(p, MONITOR_DEFAULTTONEAREST) };
        let (mut x_dpi, mut y_dpi) = (96, 96);
        if unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x_dpi, &mut y_dpi) } < 0 {
            return None;
        }
        let scale = x_dpi as f32 / 96.0;
        let scale = if scale > 0.0 { scale } else { 1.0 };
        Some((p.x as f32 / scale, p.y as f32 / scale, Some(monitor as u64)))
    }
}
