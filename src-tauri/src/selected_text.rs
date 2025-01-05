use std::{thread, time::Duration};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;


// MacOS
#[cfg(target_os = "macos")]
use tauri_plugin_dialog::MessageDialogKind;

#[cfg(target_os = "macos")]
async fn show_dialog(app: &AppHandle, message: &str, title: &str, kind: MessageDialogKind) {
    use tauri_plugin_dialog::DialogExt;
    let _ = app.dialog()
        .message(message)
        .kind(kind)
        .title(title)
        .show(|result| match result {
            true => {},
            false => {},
        });
}
#[cfg(target_os = "macos")]
pub async fn get_selected_text(app: &AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::MessageDialogKind;
    use tauri_plugin_shell::ShellExt;

    const COPY_APPLE_SCRIPT: &str = r#"tell application "System Events" to keystroke "c" using command down"#;

    let shell = app.shell();
    let output = shell.command("osascript")
                .args(&["-e", COPY_APPLE_SCRIPT])
                .output()
                .await;

    match output {
        Ok(output) => {
            if !output.status.success() {
                println!("Failed to execute AppleScript command: {:?}", output);
                show_dialog(app, "Could not have Accessibility permission! Please enable it in your MacOS setting!", "Error", MessageDialogKind::Error).await;
                return Err("Failed to execute AppleScript command".to_string());
            }
            // Wait for the clipboard to be updated
            thread::sleep(Duration::from_millis(100));

            // Get text from clipboard
            match app.clipboard().read_text() {
                Ok(selected_text) => {
                    Ok(format!("{:?}", selected_text))
                },
                Err(_) => {
                    show_dialog(app, "Could not have Accessibility permission! Please enable it in your MacOS setting!", "Error", MessageDialogKind::Error).await;
                    Err("Failed to read from clipboard".to_string())
                },
            }
        },
        Err(_) => {
            show_dialog(app, "Could not have Accessibility permission! Please enable it in your MacOS setting!", "Error", MessageDialogKind::Error).await;
            Err("Failed to execute AppleScript command".to_string())
        },
    }
}


// Windows

#[cfg(target_os = "windows")]
use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL};
#[cfg(target_os = "windows")]
use std::mem;
#[cfg(target_os = "windows")]
pub async unsafe fn get_selected_text(app: &AppHandle) -> Result<String, String> {
    thread::sleep(Duration::from_millis(100));
    call_ctrl_c();
    thread::sleep(Duration::from_millis(100));


    match app.clipboard().read_text() {
        Ok(selected_text) => {
            Ok(format!("{:?}", selected_text))
        },
        Err(_) => {
            Err("Failed to read from clipboard".to_string())
        },
    }
}

#[cfg(target_os = "windows")]
fn call_ctrl_c() {
    unsafe {
        // Create an INPUT structure for pressing the Ctrl key down
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: mem::zeroed(),
        };
        input.u.ki_mut().wVk = VK_CONTROL as u16; // VK_CONTROL is the virtual key code for the Ctrl key
        input.u.ki_mut().wScan = 0;
        input.u.ki_mut().dwFlags = 0; // 0 for key press
        input.u.ki_mut().time = 0;
        input.u.ki_mut().dwExtraInfo = 0;

        // Press the Ctrl key
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);

        // Modify the INPUT structure for pressing the C key down
        input.u.ki_mut().wVk = 0x43 as u16; // VK_C is the virtual key code for the C key
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);

        // Modify the INPUT structure to release the C key
        input.u.ki_mut().dwFlags = KEYEVENTF_KEYUP; // KEYEVENTF_KEYUP for key release
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);

        // Modify the INPUT structure to release the Ctrl key
        input.u.ki_mut().wVk = VK_CONTROL as u16;
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}



