//! The main window, laid out like Apple's Translate: a compact toolbar with
//! the mode switcher, then source and result panes side by side, each with
//! its language picker in a header and its actions in a footer.

use gpui_kit::assets::IconName;
use gpui_kit::base::SelectableText;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IndexPath, Sizable as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{caption, drag_region, hairline, panel_surface, track_appearance, truncate_middle};
use crate::refine::Mode;
use crate::settings::{DETECT_LANGUAGE, LANGUAGES};
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

const TOOLBAR_HEIGHT: f32 = 40.;
const PANE_HEADER_HEIGHT: f32 = 36.;
const PANE_FOOTER_HEIGHT: f32 = 38.;

pub struct MainView {
    model: Entity<MainModel>,
    input: Entity<TextareaState>,
    source_lang: Entity<LangSelect>,
    target_lang: Entity<LangSelect>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

fn source_items() -> Vec<SharedString> {
    std::iter::once(DETECT_LANGUAGE)
        .chain(LANGUAGES.iter().copied())
        .map(SharedString::from)
        .collect()
}

fn target_items() -> Vec<SharedString> {
    LANGUAGES.iter().map(|l| SharedString::from(*l)).collect()
}

fn index_of(items: &[SharedString], lang: &str) -> Option<IndexPath> {
    items
        .iter()
        .position(|l| l.as_ref() == lang)
        .map(|ix| IndexPath::default().row(ix))
}

/// Keeps a language picker showing `lang`.
fn sync_select(select: &Entity<LangSelect>, lang: &str, window: &mut Window, cx: &mut App) {
    let current = select.read(cx).selected_value().cloned();
    if current.as_deref() != Some(lang) {
        let lang = SharedString::from(lang.to_string());
        select.update(cx, |s, cx| s.set_selected_value(&lang, window, cx));
    }
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = AppState::get(cx).clone();
        let model = state.main.clone();
        let (text, source, target) = {
            let m = model.read(cx);
            (m.input.clone(), m.source_lang.clone(), m.target_lang.clone())
        };

        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Type or paste text")
                .default_value(text)
        });
        let sources = source_items();
        let targets = target_items();
        let source_ix = index_of(&sources, &source);
        let target_ix = index_of(&targets, &target);
        let source_lang = cx.new(|cx| SelectState::new(sources, source_ix, window, cx));
        let target_lang = cx.new(|cx| SelectState::new(targets, target_ix, window, cx));

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
                &source_lang,
                window,
                |this, _, event: &SelectEvent<Vec<SharedString>>, _, cx| {
                    if let SelectEvent::Confirm(Some(lang)) = event {
                        let lang = lang.to_string();
                        this.model.update(cx, |m, cx| m.set_source_lang(lang, cx));
                    }
                },
            ),
            cx.subscribe_in(
                &target_lang,
                window,
                |this, _, event: &SelectEvent<Vec<SharedString>>, _, cx| {
                    if let SelectEvent::Confirm(Some(lang)) = event {
                        let lang = lang.to_string();
                        this.model.update(cx, |m, cx| m.set_target_lang(lang, cx));
                    }
                },
            ),
            // Text pushed by the global shortcut, a swap, or a settings change.
            cx.observe_in(&model, window, |this, model, window, cx| {
                let (text, source, target) = {
                    let m = model.read(cx);
                    (m.input.clone(), m.source_lang.clone(), m.target_lang.clone())
                };
                if this.input.read(cx).value().as_ref() != text {
                    this.input
                        .update(cx, |input, cx| input.set_value(text, window, cx));
                }
                sync_select(&this.source_lang, &source, window, cx);
                sync_select(&this.target_lang, &target, window, cx);
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
            source_lang,
            target_lang,
            focus: cx.focus_handle(),
            _subscriptions: subs,
        }
    }

    fn on_run(&mut self, _: &Run, _: &mut Window, cx: &mut Context<Self>) {
        self.model.update(cx, |m, cx| m.run(cx));
    }

    fn on_hide(&mut self, _: &Hide, window: &mut Window, _: &mut Context<Self>) {
        windows::close_panel(window);
    }

    /// Mode switcher, a drag area and the settings button.
    fn render_toolbar(&self, mode: Mode, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h(px(TOOLBAR_HEIGHT))
            .flex_shrink_0()
            .px_2p5()
            .gap_2()
            .border_b_1()
            .border_color(hairline(cx))
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
            .child(
                Button::new("open-settings")
                    .ghost()
                    .small()
                    .icon(IconName::Settings)
                    .tooltip("Settings…")
                    .on_click(|_, _, cx| windows::show_settings(SettingsTab::General, cx)),
            )
    }

    fn pane_header(&self) -> Div {
        h_flex()
            .h(px(PANE_HEADER_HEIGHT))
            .flex_shrink_0()
            .px_2()
            .gap_1()
    }

    fn pane_footer(&self) -> Div {
        h_flex()
            .h(px(PANE_FOOTER_HEIGHT))
            .flex_shrink_0()
            .px_2()
            .gap_1()
    }

    fn render_source_pane(&self, mode: Mode, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let (chars, loading, can_swap) =
            (model.input.chars().count(), model.loading, model.can_swap());
        let has_text = !model.input.trim().is_empty();

        let header = self.pane_header().map(|this| {
            if mode == Mode::Translate {
                this.child(
                    div().w(px(150.)).child(
                        Select::new(&self.source_lang)
                            .small()
                            .appearance(false)
                            .menu_width(px(190.)),
                    ),
                )
                .child(div().flex_1())
                .child(
                    Button::new("swap-langs")
                        .ghost()
                        .xsmall()
                        .icon(IconName::ArrowLeftRight)
                        .tooltip("Swap languages")
                        .disabled(!can_swap)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.model.update(cx, |m, cx| m.swap_languages(cx))
                        })),
                )
            } else {
                this.child(div().px_1().child(caption("Original", cx).font_weight(FontWeight::MEDIUM)))
            }
        });

        let action = if loading {
            Button::new("stop")
                .small()
                .icon(IconName::Square)
                .label("Stop")
                .on_click(cx.listener(|this, _, _, cx| this.model.update(cx, |m, cx| m.stop(cx))))
        } else {
            Button::new("run")
                .primary()
                .small()
                .label(mode.label())
                .disabled(!has_text)
                .tooltip_with_action(format!("{} text", mode.label()), &Run, Some(CONTEXT))
                .on_click(cx.listener(|this, _, _, cx| this.model.update(cx, |m, cx| m.run(cx))))
        };

        let count = match chars {
            0 => String::new(),
            1 => "1 character".to_string(),
            n => format!("{n} characters"),
        };
        let footer = self
            .pane_footer()
            .child(div().pl_1().child(caption(count, cx)))
            .child(div().flex_1())
            .when(chars > 0, |this| {
                this.child(
                    Button::new("clear-input")
                        .ghost()
                        .xsmall()
                        .icon(IconName::X)
                        .tooltip("Clear")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.input
                                .update(cx, |input, cx| input.set_value("", window, cx));
                            this.model.update(cx, |m, cx| m.set_input(String::new(), cx));
                        })),
                )
            })
            .child(action);

        v_flex()
            .flex_1()
            .min_w_0()
            .child(header)
            .child(
                div().flex_1().min_h_0().px_1().child(
                    Textarea::new(&self.input)
                        .appearance(false)
                        .h_full()
                        .text_base(),
                ),
            )
            .child(footer)
    }

    fn render_result_pane(&self, mode: Mode, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let theme = cx.theme();
        let settings = AppState::get(cx).settings.read(cx);
        let engine = format!(
            "{} · {}",
            settings.provider_info().label,
            truncate_middle(&settings.model, 24)
        );

        let header = self.pane_header().map(|this| match mode {
            Mode::Translate => this.child(
                div().w(px(150.)).child(
                    Select::new(&self.target_lang)
                        .small()
                        .appearance(false)
                        .menu_width(px(190.)),
                ),
            ),
            Mode::Correct | Mode::Refine => this.child(
                div().px_1().child(
                    caption(if mode == Mode::Correct { "Corrected" } else { "Refined" }, cx)
                        .font_weight(FontWeight::MEDIUM),
                ),
            ),
        });

        let body: AnyElement = if model.loading {
            v_flex()
                .gap_2p5()
                .pt_1()
                .child(Skeleton::new().h_3().w_full().rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.9)).rounded_sm())
                .child(Skeleton::new().h_3().w(relative(0.6)).rounded_sm())
                .into_any_element()
        } else {
            match model.result() {
                Some(Ok(text)) => div()
                    .text_base()
                    .line_height(relative(1.5))
                    .child(SelectableText::new("result-text", text.clone()))
                    .into_any_element(),
                Some(Err(err)) => div()
                    .text_sm()
                    .text_color(theme.danger)
                    .child(SelectableText::new("result-error", err.clone()))
                    .into_any_element(),
                None => div()
                    .text_base()
                    .text_color(theme.muted_foreground.opacity(0.7))
                    .child(match mode {
                        Mode::Translate => "Translation",
                        Mode::Correct => "Corrected text",
                        Mode::Refine => "Refined text",
                    })
                    .into_any_element(),
            }
        };

        let copy_text = match (model.loading, model.result()) {
            (false, Some(Ok(text))) => Some(text.clone()),
            _ => None,
        };
        let footer = self
            .pane_footer()
            .child(div().pl_1().min_w_0().child(caption(engine, cx)))
            .child(div().flex_1())
            .when_some(copy_text, |this, text| {
                this.child(Clipboard::new("copy-result").value(text).tooltip("Copy"))
            });

        v_flex()
            .flex_1()
            .min_w_0()
            .bg(theme.muted.opacity(0.35))
            .child(header)
            .child(
                div()
                    .id("result-scroll")
                    .flex_1()
                    .min_h_0()
                    .px_3()
                    .py_1()
                    .overflow_y_scroll()
                    .child(body),
            )
            .child(footer)
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for MainView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mode = self.model.read(cx).mode;
        panel_surface(cx)
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::on_run))
            .on_action(cx.listener(Self::on_hide))
            .child(self.render_toolbar(mode, cx))
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_stretch()
                    .child(self.render_source_pane(mode, cx))
                    .child(div().w(px(1.)).h_full().bg(hairline(cx)))
                    .child(self.render_result_pane(mode, cx)),
            )
    }
}
