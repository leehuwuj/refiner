//! The compact popup shown next to the cursor (global shortcut or the
//! selection icon): a translation into the preferred language and a grammar
//! correction, styled like a macOS popover.

use gpui_kit::assets::IconName;
use gpui_kit::base::SelectableText;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{ActiveTheme as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{caption, drag_region, hairline, panel_surface, track_appearance};
use crate::state::{AppState, Outcome, PopupModel};
use crate::windows;

const CONTEXT: &str = "RefinerPopup";

gpui_kit::actions!(refiner_popup, [ClosePopup, NextTab]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", ClosePopup, Some(CONTEXT)),
        KeyBinding::new("left", NextTab, Some(CONTEXT)),
        KeyBinding::new("right", NextTab, Some(CONTEXT)),
        KeyBinding::new("tab", NextTab, Some(CONTEXT)),
    ]);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PopupTab {
    Translation,
    Correction,
}

pub struct PopupView {
    model: Entity<PopupModel>,
    tab: PopupTab,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl PopupView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let model = AppState::get(cx).popup.clone();
        let mut subs = vec![
            cx.observe(&model, |_, _, cx| cx.notify()),
            windows::close_on_blur(window, cx),
        ];
        subs.extend(track_appearance(window, cx));
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Self {
            model,
            tab: PopupTab::Translation,
            focus,
            _subscriptions: subs,
        }
    }

    fn body(&self, outcome: Option<&Outcome>, loading: bool, cx: &App) -> AnyElement {
        let theme = cx.theme();
        if loading {
            return v_flex()
                .gap_2()
                .pt_1()
                .child(Skeleton::new().h_3().w_full().rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.8)).rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.5)).rounded_sm())
                .into_any_element();
        }
        match outcome {
            Some(Ok(text)) => div()
                .text_base()
                .line_height(relative(1.45))
                .child(SelectableText::new("popup-text", text.clone()))
                .into_any_element(),
            Some(Err(err)) => div()
                .text_sm()
                .text_color(theme.danger)
                .child(SelectableText::new("popup-error", err.clone()))
                .into_any_element(),
            None => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(caption(
                    if cfg!(target_os = "macos") {
                        "Select text and press ⌘E"
                    } else {
                        "Select text and press Ctrl+E"
                    },
                    cx,
                ))
                .into_any_element(),
        }
    }
}

impl Focusable for PopupView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for PopupView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let (outcome, loading, selected, footer_label) = match self.tab {
            PopupTab::Translation => (
                model.translation.clone(),
                model.translate_loading(),
                0,
                AppState::get(cx).settings.read(cx).preferred_lang.clone(),
            ),
            PopupTab::Correction => (
                model.correction.clone(),
                model.correct_loading(),
                1,
                "Grammar".to_string(),
            ),
        };
        let copy = match (&outcome, loading) {
            (Some(Ok(text)), false) => Some(text.clone()),
            _ => None,
        };

        panel_surface(cx)
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|_, _: &ClosePopup, window, _| window.remove_window()))
            .on_action(cx.listener(|this, _: &NextTab, _, cx| {
                this.tab = match this.tab {
                    PopupTab::Translation => PopupTab::Correction,
                    PopupTab::Correction => PopupTab::Translation,
                };
                cx.notify();
            }))
            .child(
                h_flex()
                    .h(px(36.))
                    .flex_shrink_0()
                    .px_2()
                    .gap_2()
                    .border_b_1()
                    .border_color(hairline(cx))
                    .child(
                        TabBar::new("popup-tabs")
                            .segmented()
                            .xsmall()
                            .selected_index(selected)
                            .child(Tab::new().label("Translation"))
                            .child(Tab::new().label("Correction"))
                            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                this.tab = if *ix == 0 {
                                    PopupTab::Translation
                                } else {
                                    PopupTab::Correction
                                };
                                cx.notify();
                            })),
                    )
                    .child(drag_region("popup-drag").flex_1().h_full())
                    .child(
                        Button::new("open-main")
                            .ghost()
                            .xsmall()
                            .icon(IconName::ExternalLink)
                            .tooltip("Open in Refiner")
                            .on_click(|_, window, cx| {
                                window.remove_window();
                                windows::show_main(cx);
                            }),
                    ),
            )
            .child(
                div()
                    .id("popup-scroll")
                    .flex_1()
                    .min_h_0()
                    .px_3()
                    .py_2()
                    .overflow_y_scroll()
                    .child(self.body(outcome.as_ref(), loading, cx)),
            )
            .child(
                h_flex()
                    .h(px(30.))
                    .flex_shrink_0()
                    .px_3()
                    .gap_1()
                    .child(caption(footer_label, cx))
                    .child(div().flex_1())
                    .when_some(copy, |this, text| {
                        this.child(Clipboard::new("copy-popup").value(text).tooltip("Copy"))
                    }),
            )
    }
}
