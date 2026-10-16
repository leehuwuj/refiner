//! The main window: Translate / Correct / Refine with input and output side
//! by side.

use gpui_kit::assets::IconName;
use gpui_kit::base::SelectableText;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{ActiveTheme as _, IndexPath, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{chip, drag_region, glass_surface, panel, track_appearance, truncate_middle};
use crate::refine::Mode;
use crate::settings::LANGUAGES;
use crate::state::{AppState, MainModel};
use crate::ui::settings_view::SettingsTab;
use crate::windows;

pub const CONTEXT: &str = "RefinerMain";

gpui_kit::actions!(refiner_main, [Run, Hide]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-enter", Run, Some(CONTEXT)),
        KeyBinding::new("escape", Hide, Some(CONTEXT)),
    ]);
}

type LangSelect = SelectState<Vec<SharedString>>;

pub struct MainView {
    model: Entity<MainModel>,
    input: Entity<TextareaState>,
    target_lang: Entity<LangSelect>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

fn lang_items() -> Vec<SharedString> {
    LANGUAGES.iter().map(|l| SharedString::from(*l)).collect()
}

fn lang_index(lang: &str) -> Option<IndexPath> {
    LANGUAGES
        .iter()
        .position(|l| *l == lang)
        .map(|ix| IndexPath::default().row(ix))
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = AppState::get(cx).clone();
        let model = state.main.clone();
        let (text, target) = {
            let m = model.read(cx);
            (m.input.clone(), m.target_lang.clone())
        };

        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Paste or type text here…\n\nCtrl+Enter to process.")
                .default_value(text)
        });
        let target_lang =
            cx.new(|cx| SelectState::new(lang_items(), lang_index(&target), window, cx));

        let mut subs = vec![
            cx.subscribe_in(&input, window, |this, input, event: &InputEvent, _, cx| {
                match event {
                    InputEvent::Change => {
                        let value = input.read(cx).value().to_string();
                        this.model.update(cx, |m, cx| m.set_input(value, cx));
                    }
                    InputEvent::PressEnter { secondary: true, .. } => {
                        this.model.update(cx, |m, cx| m.run(cx));
                    }
                    _ => {}
                }
            }),
            cx.subscribe_in(
                &target_lang,
                window,
                |this, _, event: &SelectEvent<Vec<SharedString>>, _, cx| {
                    let SelectEvent::Confirm(Some(lang)) = event else {
                        return;
                    };
                    let lang = lang.to_string();
                    this.model.update(cx, |m, cx| m.set_target_lang(lang, cx));
                },
            ),
            // Text pushed by the global shortcut, or a language swap.
            cx.observe_in(&model, window, |this, model, window, cx| {
                let (text, target) = {
                    let m = model.read(cx);
                    (m.input.clone(), m.target_lang.clone())
                };
                if this.input.read(cx).value().as_ref() != text {
                    this.input
                        .update(cx, |input, cx| input.set_value(text, window, cx));
                }
                let selected = this.target_lang.read(cx).selected_value().cloned();
                if selected.as_deref() != Some(target.as_str()) {
                    this.target_lang.update(cx, |s, cx| {
                        s.set_selected_value(&SharedString::from(target), window, cx)
                    });
                }
                cx.notify();
            }),
            cx.observe(&state.settings, |_, _, cx| cx.notify()),
            windows::close_on_blur(window, cx),
        ];
        subs.extend(track_appearance(window, cx));

        input.update(cx, |input, cx| input.focus(window, cx));

        Self {
            model,
            input,
            target_lang,
            focus: cx.focus_handle(),
            _subscriptions: subs,
        }
    }

    fn on_run(&mut self, _: &Run, _: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |m, cx| m.run(cx));
    }

    fn on_hide(&mut self, _: &Hide, window: &mut Window, _: &mut Context<Self>) {
        window.remove_window();
    }

    fn render_header(&self, mode: Mode, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = AppState::get(cx).settings.read(cx);
        let model_name = settings.model.clone();
        h_flex()
            .h(px(48.))
            .px_3()
            .gap_2()
            .flex_shrink_0()
            .child(
                TabBar::new("modes")
                    .segmented()
                    .small()
                    .selected_index(mode.index())
                    .children(Mode::ALL.map(|m| Tab::new().label(m.label())))
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        if let Some(mode) = Mode::ALL.get(*ix).copied() {
                            this.model.update(cx, |m, cx| m.set_mode(mode, cx));
                        }
                    })),
            )
            .child(drag_region("main-drag").flex_1().h_full())
            .when(!model_name.is_empty(), |this| {
                this.child(chip(truncate_middle(&model_name, 22), cx))
            })
            .child(
                Button::new("open-settings")
                    .ghost()
                    .small()
                    .icon(IconName::Settings)
                    .tooltip("Settings")
                    .on_click(|_, _, cx| windows::show_settings(SettingsTab::General, cx)),
            )
    }

    fn render_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let chars = model.input.chars().count();
        let theme = cx.theme();
        panel(cx)
            .flex_1()
            .min_w_0()
            .child(
                div().flex_1().min_h_0().p_1().child(
                    Textarea::new(&self.input)
                        .appearance(false)
                        .h_full()
                        .text_sm(),
                ),
            )
            .child(
                h_flex()
                    .h(px(28.))
                    .px_3()
                    .justify_between()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(if chars > 0 {
                        format!("{chars} chars")
                    } else {
                        String::new()
                    })
                    .when(chars > 0, |this| {
                        this.child(
                            Button::new("clear-input")
                                .text()
                                .xsmall()
                                .label("Clear")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.input
                                        .update(cx, |input, cx| input.set_value("", window, cx));
                                    this.model.update(cx, |m, cx| m.set_input(String::new(), cx));
                                })),
                        )
                    }),
            )
    }

    fn render_action(&self, loading: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let button = if loading {
            Button::new("stop")
                .icon(IconName::Square)
                .tooltip("Stop")
                .on_click(cx.listener(|this, _, _, cx| this.model.update(cx, |m, cx| m.stop(cx))))
        } else {
            Button::new("run")
                .primary()
                .icon(IconName::ArrowRight)
                .tooltip("Run (Ctrl+Enter)")
                .on_click(cx.listener(|this, _, _, cx| this.model.update(cx, |m, cx| m.run(cx))))
        };
        v_flex()
            .w(px(56.))
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .child(button.size(px(42.)).rounded_full())
    }

    fn render_output(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let theme = cx.theme();
        let body: AnyElement = if model.loading {
            v_flex()
                .gap_2()
                .pt_1()
                .child(Skeleton::new().h_3().w_full().rounded_sm())
                .child(Skeleton::new().h_3().w_full().rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.85)).rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.6)).rounded_sm())
                .into_any_element()
        } else {
            match model.result() {
                Some(Ok(text)) => div()
                    .text_sm()
                    .line_height(relative(1.6))
                    .pr_8()
                    .child(SelectableText::new("result-text", text.clone()))
                    .into_any_element(),
                Some(Err(err)) => div()
                    .text_sm()
                    .text_color(theme.danger)
                    .child(SelectableText::new("result-error", err.clone()))
                    .into_any_element(),
                None => div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("Result will appear here")
                    .into_any_element(),
            }
        };
        let copy_text = match (model.loading, model.result()) {
            (false, Some(Ok(text))) => Some(text.clone()),
            _ => None,
        };
        panel(cx).flex_1().min_w_0().relative().child(
            div()
                .id("result-scroll")
                .flex_1()
                .min_h_0()
                .px_3p5()
                .py_3()
                .overflow_y_scroll()
                .child(body),
        )
        .when_some(copy_text, |this, text| {
            this.child(
                div()
                    .absolute()
                    .top_2()
                    .right_2()
                    .child(Clipboard::new("copy-result").value(text).tooltip("Copy")),
            )
        })
    }

    fn render_footer(&self, mode: Mode, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let source = model.source_lang.clone();
        let provider = AppState::get(cx).settings.read(cx).provider_info().label;
        let hint = if cfg!(target_os = "macos") { "⌘⏎" } else { "Ctrl ⏎" };
        h_flex()
            .h(px(44.))
            .px_3p5()
            .flex_shrink_0()
            .justify_between()
            .child(h_flex().flex_1().child(chip(hint, cx)))
            .child(
                h_flex()
                    .gap_1p5()
                    .when(mode == Mode::Translate, |this| {
                        this.child(chip(source, cx))
                            .child(
                                Button::new("swap-langs")
                                    .ghost()
                                    .small()
                                    .icon(IconName::ArrowLeftRight)
                                    .tooltip("Swap languages")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.model.update(cx, |m, cx| m.swap_languages(cx))
                                    })),
                            )
                            .child(
                                div().w(px(132.)).child(
                                    Select::new(&self.target_lang)
                                        .small()
                                        .menu_width(px(160.)),
                                ),
                            )
                    }),
            )
            .child(h_flex().flex_1().justify_end().child(chip(provider, cx)))
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for MainView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (mode, loading) = {
            let m = self.model.read(cx);
            (m.mode, m.loading)
        };
        glass_surface(cx)
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::on_run))
            .on_action(cx.listener(Self::on_hide))
            .child(self.render_header(mode, cx))
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .px_2p5()
                    .items_stretch()
                    .child(self.render_input(cx))
                    .child(self.render_action(loading, cx))
                    .child(self.render_output(cx)),
            )
            .child(self.render_footer(mode, cx))
    }
}
