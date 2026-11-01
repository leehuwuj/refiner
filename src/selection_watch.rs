//! Floating selection icon: after the user selects text with the
//! mouse or keyboard in another app, show a button above the text that opens
//! the compact popup for that text.
//!
//! Each platform observes accessibility selection changes and gates display
//! until the mouse or keyboard selection gesture has completed.
//! There is deliberately no ⌘C fallback here: that would overwrite the
//! clipboard on every mouse-up. Apps that don't expose their selection simply
//! get no icon (⌘E still works there).

use gpui_kit::*;

/// Starts watching when the setting is on, and follows later changes to it.
pub fn init(cx: &mut App) {
    #[cfg(target_os = "macos")]
    crate::macos_selection_watch::init(cx);
    #[cfg(target_os = "windows")]
    crate::windows_selection_watch::init(cx);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = cx;
}
