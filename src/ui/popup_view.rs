//! The compact popup shown next to the cursor by the global shortcut:
//! a translation into the preferred language and a grammar correction.

use gpui_kit::assets::IconName;
use gpui_kit::base::SelectableText;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{ActiveTheme as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{drag_region, glass_surface, panel, track_appearance};
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

    fn render_result(
        &self,
        label: &'static str,
        outcome: Option<&Outcome>,
        loading: bool,
        cx: &App,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let copy = match outcome {
            Some(Ok(text)) if !loading => Some(text.clone()),
            _ => None,
        };
        let body: AnyElement = if loading {
            v_flex()
                .gap_2()
                .child(Skeleton::new().h_3().w_full().rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.8)).rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.55)).rounded_sm())
                .into_any_element()
        } else {
            match outcome {
                Some(Ok(text)) => div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .line_height(relative(1.5))
                    .child(SelectableText::new(SharedString::from(format!("popup-text-{label}")), text.clone()))
                    .into_any_element(),
                Some(Err(err)) => div()
                    .text_xs()
                    .text_color(theme.danger)
                    .child(SelectableText::new(SharedString::from(format!("popup-error-{label}")), err.clone()))
                    .into_any_element(),
                None => div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("Waiting for selection…")
                    .into_any_element(),
            }
        };
        panel(cx)
            .flex_1()
            .min_h_0()
            .mx_2p5()
            .mb_2()
            .child(
                h_flex()
                    .h(px(30.))
                    .px_2p5()
                    .justify_between()
                    .flex_shrink_0()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.muted_foreground)
                            .child(label),
                    )
                    .when_some(copy, |this, text| {
                        this.child(Clipboard::new("copy-popup").value(text).tooltip("Copy"))
                    }),
            )
            .child(
                div()
                    .id("popup-scroll")
                    .flex_1()
                    .min_h_0()
                    .px_3()
                    .pb_2p5()
                    .overflow_y_scroll()
                    .child(body),
            )
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
        let result = match self.tab {
            PopupTab::Translation => self.render_result(
                "TRANSLATION",
                model.translation.as_ref(),
                model.translate_loading(),
                cx,
            ),
            PopupTab::Correction => self.render_result(
                "CORRECTION",
                model.correction.as_ref(),
                model.correct_loading(),
                cx,
            ),
        };
        let selected = match self.tab {
            PopupTab::Translation => 0,
            PopupTab::Correction => 1,
        };

        glass_surface(cx)
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
                    .h(px(42.))
                    .px_2p5()
                    .gap_2()
                    .flex_shrink_0()
                    .child(
                        TabBar::new("popup-tabs")
                            .segmented()
                            .xsmall()
                            .selected_index(selected)
                            .child(Tab::new().icon(IconName::Languages).aria_label("Translate"))
                            .child(Tab::new().icon(IconName::Sparkles).aria_label("Correct"))
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
                            .tooltip("Open Refiner")
                            .on_click(|_, window, cx| {
                                window.remove_window();
                                windows::show_main(cx);
                            }),
                    ),
            )
            .child(result)
    }
}
