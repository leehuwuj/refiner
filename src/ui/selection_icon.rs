//! The small floating button shown next to a mouse selection. Clicking it
//! opens the compact popup for the selected text.

use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _};
use gpui_kit::*;

use crate::state::AppState;
use crate::windows;

/// Hide the icon if it isn't used within this long.
const LIFETIME: Duration = Duration::from_secs(5);

pub struct SelectionIcon {
    text: String,
    hovered: bool,
    _expire: Task<()>,
}

impl SelectionIcon {
    pub fn new(text: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let expire = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(LIFETIME).await;
            // Keep it while the pointer rests on it.
            loop {
                let hovered = this.update(cx, |this, _| this.hovered).unwrap_or(false);
                if !hovered {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
            }
            let _ = cx.update(|window, _| window.remove_window());
        });
        Self {
            text,
            hovered: false,
            _expire: expire,
        }
    }
}

impl Render for SelectionIcon {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id("selection-icon")
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .border_1()
            .border_color(theme.border)
            .bg(if self.hovered {
                theme.primary
            } else {
                theme.background.opacity(0.92)
            })
            .text_color(if self.hovered {
                theme.primary_foreground
            } else {
                theme.foreground
            })
            .cursor_pointer()
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                this.hovered = *hovered;
                cx.notify();
            }))
            // Mouse-down, not click: the panel never becomes key, so act on
            // the first event it receives.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    let text = std::mem::take(&mut this.text);
                    window.remove_window();
                    let popup = AppState::get(cx).popup.clone();
                    popup.update(cx, |p, cx| p.run(text, cx));
                    windows::show_popup(cx);
                }),
            )
            .child(Icon::new(IconName::Languages).xsmall())
    }
}
