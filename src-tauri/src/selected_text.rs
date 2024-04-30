use std::{thread, time::Duration};

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_shell::ShellExt;


pub async fn get_selected_text(app: &AppHandle) -> Result<String, String> {
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
                return Err("Failed to execute AppleScript command".to_string());
            }
            // Wait for the clipboard to be updated
            thread::sleep(Duration::from_millis(100));

            // Get text from clipboard
            match app.clipboard().read() {
                Ok(selected_text) => {
                    println!("Selected text: {:?}", selected_text);
                    Ok(format!("{:?}", selected_text))
                },
                Err(_) => Err("Failed to read from clipboard".to_string()),
            }
        },
        Err(_) => Err("Failed to execute AppleScript command".to_string()),
    }
}