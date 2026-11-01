//! Views and shared styling.

pub mod main_view;
pub mod popup_view;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub mod selection_icon;
pub mod settings_view;

use gpui_kit::component::{ActiveTheme as _, Theme, ThemeMode};
use gpui_kit::*;

use crate::settings::ThemePref;
use crate::state::AppState;

/// Base font size before the user's text-size scale.
const BASE_FONT_SIZE: f32 = 13.;

/// Applies the saved theme and text size.
pub fn apply_appearance(window: &mut Window, cx: &mut App) {
    let settings = AppState::get(cx).settings.read(cx);
    let (theme, scale) = (settings.theme, settings.text_size.scale());
    match theme {
        ThemePref::System => Theme::sync_system_appearance(Some(window), cx),
        ThemePref::Dark => Theme::change(ThemeMode::Dark, Some(window), cx),
        ThemePref::Light => Theme::change(ThemeMode::Light, Some(window), cx),
    }
    let size = px(BASE_FONT_SIZE * scale);
    if Theme::global(cx).font_size != size {
        Theme::update(cx, |t| t.font_size = size);
    }
}

/// Keeps a window's appearance in sync with settings and, for the System
/// theme, with the OS. Call from a view constructor.
pub fn track_appearance<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> [Subscription; 2] {
    let settings = AppState::get(cx).settings.clone();
    [
        cx.observe_in(&settings, window, |_, _, window, cx| {
            apply_appearance(window, cx);
            cx.notify();
        }),
        cx.observe_window_appearance(window, |_, window, cx| {
            if AppState::get(cx).settings.read(cx).theme == ThemePref::System {
                apply_appearance(window, cx);
            }
        }),
    ]
}

/// Corner radius of the floating panels (macOS popovers use ~10–12pt).
pub const PANEL_RADIUS: f32 = 12.;

/// The translucent surface of the floating panels. The window is created
/// with a blurred background, so a partly transparent fill reads as macOS
/// vibrancy.
pub fn panel_surface(cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .size_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .rounded(px(PANEL_RADIUS))
        .border_1()
        .border_color(hairline(cx))
        .bg(theme.background.opacity(0.78))
        .text_color(theme.foreground)
}

/// The thin separator colour used between toolbar, panes and footers.
pub fn hairline(cx: &App) -> Hsla {
    cx.theme().border.opacity(0.55)
}

/// Small secondary text (status lines, captions).
pub fn caption(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .whitespace_nowrap()
        .child(text.into())
}

/// Shortens long model names in the middle: `qwen3:30b-a3b-…-q4_K_M`.
pub fn truncate_middle(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        return s.to_string();
    }
    let head = (max - 1).div_ceil(2);
    let tail = (max - 1) / 2;
    let mut out: String = chars[..head].iter().collect();
    out.push('…');
    out.extend(&chars[chars.len() - tail..]);
    out
}

/// A drag handle area: dragging it moves the (titlebar-less) window.
pub fn drag_region(id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
}

#[cfg(test)]
mod tests {
    use super::truncate_middle;

    #[test]
    fn truncates_in_the_middle() {
        assert_eq!(truncate_middle("short", 22), "short");
        let t = truncate_middle("qwen3:30b-a3b-instruct-2507-q4_K_M", 22);
        assert_eq!(t.chars().count(), 22);
        assert!(t.starts_with("qwen3:30b-") && t.ends_with("q4_K_M"));
    }
}
