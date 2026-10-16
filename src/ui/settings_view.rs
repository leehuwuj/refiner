//! The Settings window, laid out like macOS System Settings: icon tabs in a
//! toolbar, then grouped rows with the label on the left and the control on
//! the right. Changes apply immediately; there is no Save button.

use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_kit::component::progress::Progress;
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, IndexPath, Sizable as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{caption, hairline, track_appearance};
use crate::analysis::{Phase, format_report_name};
use crate::history;
use crate::platform;
use crate::refine::Mode;
use crate::settings::{
    LANGUAGES, PROVIDERS, Settings, ShortcutWindow, TextSize, ThemePref, data_dir, provider_info,
};
use crate::state::{AnalysisModel, AppState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Model,
    Prompts,
    Analysis,
}

impl SettingsTab {
    const ALL: [Self; 4] = [Self::General, Self::Model, Self::Prompts, Self::Analysis];

    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Model => "Model",
            Self::Prompts => "Prompts",
            Self::Analysis => "Analysis",
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::General => IconName::SlidersHorizontal,
            Self::Model => IconName::Cpu,
            Self::Prompts => IconName::MessageSquare,
            Self::Analysis => IconName::ChartColumn,
        }
    }
}

const DAY_RANGES: [(Option<u32>, &str); 5] = [
    (Some(7), "Last 7 days"),
    (Some(14), "Last 14 days"),
    (Some(30), "Last 30 days"),
    (Some(90), "Last 90 days"),
    (None, "All time"),
];

/// Width of the control column in form rows.
const CONTROL_WIDTH: f32 = 240.;
/// Typing in a text field saves after this pause.
const SAVE_DELAY: Duration = Duration::from_millis(400);

type LabelSelect = SelectState<Vec<SharedString>>;

pub struct SettingsView {
    tab: SettingsTab,
    /// Choices made with pickers and switches; text fields are read on save.
    draft: Settings,
    provider: Entity<LabelSelect>,
    shortcut_window: Entity<LabelSelect>,
    preferred_lang: Entity<LabelSelect>,
    theme: Entity<LabelSelect>,
    text_size: Entity<LabelSelect>,
    day_range: Entity<LabelSelect>,
    model: Entity<InputState>,
    base_url: Entity<InputState>,
    api_key: Entity<InputState>,
    prompts: [Entity<TextareaState>; 3],
    analysis: Entity<AnalysisModel>,
    confirm_clear: bool,
    pending_save: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

fn labels(items: impl IntoIterator<Item = &'static str>) -> Vec<SharedString> {
    items.into_iter().map(SharedString::from).collect()
}

fn row_index(ix: Option<usize>) -> Option<IndexPath> {
    ix.map(|ix| IndexPath::default().row(ix))
}

fn url_placeholder(provider: &str) -> &'static str {
    if provider == "ollama" {
        "http://localhost:11434"
    } else {
        "Default"
    }
}

type Confirm = SelectEvent<Vec<SharedString>>;

impl SettingsView {
    pub fn new(tab: SettingsTab, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = AppState::get(cx).clone();
        let s = state.settings.read(cx).clone();

        let select = |items: Vec<SharedString>,
                      ix: Option<usize>,
                      window: &mut Window,
                      cx: &mut Context<Self>| {
            cx.new(|cx| SelectState::new(items, row_index(ix), window, cx))
        };
        let provider = select(
            labels(PROVIDERS.iter().map(|p| p.label)),
            PROVIDERS.iter().position(|p| p.id == s.provider),
            window,
            cx,
        );
        let shortcut_window = select(
            labels(ShortcutWindow::ALL.map(ShortcutWindow::label)),
            ShortcutWindow::ALL.iter().position(|w| *w == s.shortcut_window),
            window,
            cx,
        );
        let preferred_lang = select(
            labels(LANGUAGES.iter().copied()),
            LANGUAGES.iter().position(|l| *l == s.preferred_lang),
            window,
            cx,
        );
        let theme = select(
            labels(ThemePref::ALL.map(ThemePref::label)),
            ThemePref::ALL.iter().position(|t| *t == s.theme),
            window,
            cx,
        );
        let text_size = select(
            labels(TextSize::ALL.map(TextSize::label)),
            TextSize::ALL.iter().position(|t| *t == s.text_size),
            window,
            cx,
        );
        let day_range = select(labels(DAY_RANGES.map(|(_, l)| l)), Some(2), window, cx);

        let model = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Model name")
                .default_value(s.model.clone())
        });
        let base_url = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(url_placeholder(&s.provider))
                .default_value(s.model_url.clone())
        });
        let api_key = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("Required")
                .default_value(s.api_key.clone())
        });
        let prompt = |value: &str, window: &mut Window, cx: &mut Context<Self>| {
            let value = value.to_string();
            cx.new(|cx| {
                TextareaState::new(window, cx)
                    .rows(4)
                    .placeholder("Default prompt")
                    .default_value(value)
            })
        };
        let prompts = [
            prompt(&s.prompt_translate, window, cx),
            prompt(&s.prompt_correct, window, cx),
            prompt(&s.prompt_refine, window, cx),
        ];

        let mut subs = vec![
            cx.subscribe_in(&provider, window, |this, _, e: &Confirm, window, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                let Some(p) = PROVIDERS.iter().find(|p| p.label == label.as_ref()) else {
                    return;
                };
                if this.draft.provider != p.id {
                    this.draft.provider = p.id.to_string();
                    let first = p.models.first().copied().unwrap_or_default();
                    this.model.update(cx, |m, cx| m.set_value(first, window, cx));
                    this.base_url.update(cx, |u, cx| {
                        u.set_placeholder(url_placeholder(p.id), window, cx)
                    });
                    this.save(cx);
                }
            }),
            cx.subscribe_in(&shortcut_window, window, |this, _, e: &Confirm, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                if let Some(w) = ShortcutWindow::ALL.into_iter().find(|w| w.label() == label.as_ref()) {
                    this.draft.shortcut_window = w;
                    this.save(cx);
                }
            }),
            cx.subscribe_in(&preferred_lang, window, |this, _, e: &Confirm, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                this.draft.preferred_lang = label.to_string();
                this.save(cx);
            }),
            cx.subscribe_in(&theme, window, |this, _, e: &Confirm, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                if let Some(t) = ThemePref::ALL.into_iter().find(|t| t.label() == label.as_ref()) {
                    this.draft.theme = t;
                    this.save(cx);
                }
            }),
            cx.subscribe_in(&text_size, window, |this, _, e: &Confirm, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                if let Some(t) = TextSize::ALL.into_iter().find(|t| t.label() == label.as_ref()) {
                    this.draft.text_size = t;
                    this.save(cx);
                }
            }),
            cx.observe(&state.analysis, |_, _, cx| cx.notify()),
        ];
        for input in [&model, &base_url, &api_key] {
            subs.push(cx.subscribe(input, |this, _, e: &InputEvent, cx| {
                if matches!(e, InputEvent::Change) {
                    this.save_later(cx);
                }
            }));
        }
        for prompt in &prompts {
            subs.push(cx.subscribe(prompt, |this, _, e: &InputEvent, cx| {
                if matches!(e, InputEvent::Change) {
                    this.save_later(cx);
                }
            }));
        }
        subs.extend(track_appearance(window, cx));
        // Re-check Accessibility access when the user comes back from System Settings.
        subs.push(cx.observe_window_activation(window, |_, window, cx| {
            if window.is_window_active() {
                cx.notify();
            }
        }));
        // Closing the window must not drop an edit still waiting to be saved.
        subs.push(cx.on_release(|this, cx| {
            if this.pending_save.take().is_some() {
                let next = this.collect(cx);
                if &next != AppState::get(cx).settings.read(cx)
                    && let Err(err) = AppState::save_settings(next, cx)
                {
                    eprintln!("refiner: failed to save settings: {err:#}");
                }
            }
        }));

        state.analysis.update(cx, |a, cx| a.refresh(cx));
        window.set_window_title(tab.label());

        Self {
            tab,
            draft: s,
            provider,
            shortcut_window,
            preferred_lang,
            theme,
            text_size,
            day_range,
            model,
            base_url,
            api_key,
            prompts,
            analysis: state.analysis.clone(),
            confirm_clear: false,
            pending_save: None,
            _subscriptions: subs,
        }
    }

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        self.tab = tab;
        self.confirm_clear = false;
        if tab == SettingsTab::Analysis {
            self.analysis.update(cx, |a, cx| a.refresh(cx));
        }
        cx.notify();
    }

    /// The draft plus the current text-field values.
    fn collect(&self, cx: &App) -> Settings {
        let mut next = self.draft.clone();
        next.model = self.model.read(cx).value().trim().to_string();
        next.model_url = self.base_url.read(cx).value().trim().to_string();
        next.api_key = self.api_key.read(cx).value().trim().to_string();
        let [t, c, r] = &self.prompts;
        next.prompt_translate = t.read(cx).value().trim().to_string();
        next.prompt_correct = c.read(cx).value().trim().to_string();
        next.prompt_refine = r.read(cx).value().trim().to_string();
        next.history_enabled = AppState::get(cx).settings.read(cx).history_enabled;
        next
    }

    /// Saves now if anything changed.
    fn save(&mut self, cx: &mut Context<Self>) {
        self.pending_save = None;
        let next = self.collect(cx);
        if &next != AppState::get(cx).settings.read(cx)
            && let Err(err) = AppState::save_settings(next.clone(), cx)
        {
            eprintln!("refiner: failed to save settings: {err:#}");
        }
        self.draft = next;
        cx.notify();
    }

    /// Saves after a pause in typing.
    fn save_later(&mut self, cx: &mut Context<Self>) {
        self.pending_save = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            let _ = this.update(cx, |this, cx| this.save(cx));
        }));
    }

    fn set_history_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let mut saved = AppState::settings(cx);
        saved.history_enabled = enabled;
        self.draft.history_enabled = enabled;
        if let Err(err) = AppState::save_settings(saved, cx) {
            self.analysis
                .update(cx, |a, cx| a.flash(true, err.to_string(), cx));
        }
        cx.notify();
    }

    fn selected_days(&self, cx: &App) -> Option<u32> {
        let label = self.day_range.read(cx).selected_value().cloned();
        DAY_RANGES
            .iter()
            .find(|(_, l)| label.as_deref() == Some(*l))
            .and_then(|(d, _)| *d)
    }

    // ── Toolbar ────────────────────────────────────────────────────────────

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        h_flex()
            .w_full()
            .justify_center()
            .gap_1()
            .py_1p5()
            .flex_shrink_0()
            .border_b_1()
            .border_color(hairline(cx))
            .bg(theme.title_bar)
            .children(SettingsTab::ALL.into_iter().map(|tab| {
                let active = tab == self.tab;
                v_flex()
                    .id(tab.label())
                    .w(px(76.))
                    .py_1()
                    .gap_0p5()
                    .items_center()
                    .rounded(px(7.))
                    .cursor_pointer()
                    .text_color(if active { theme.primary } else { theme.muted_foreground })
                    .when(active, |this| this.bg(theme.secondary))
                    .hover(|this| this.bg(theme.secondary.opacity(0.6)))
                    .child(Icon::new(tab.icon()).size_5())
                    .child(div().text_xs().child(tab.label()))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        window.set_window_title(tab.label());
                        this.set_tab(tab, cx);
                    }))
            }))
    }

    // ── General ────────────────────────────────────────────────────────────

    fn render_general(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let trusted = platform::selection_access(false);
        let shortcut = if cfg!(target_os = "macos") { "⌘E" } else { "Ctrl+E" };

        let access_control: AnyElement = if trusted {
            h_flex()
                .gap_1p5()
                .text_sm()
                .text_color(theme.success)
                .child(Icon::new(IconName::Check).small())
                .child("Granted")
                .into_any_element()
        } else {
            Button::new("open-accessibility")
                .small()
                .label("Open Privacy Settings…")
                .on_click(|_, _, cx| {
                    cx.open_url(
                        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                    )
                })
                .into_any_element()
        };

        v_flex()
            .gap_5()
            .child(group(
                Some("Appearance"),
                vec![
                    row("Theme", None, select_box(&self.theme), cx),
                    row("Text size", None, select_box(&self.text_size), cx),
                ],
                cx,
            ))
            .child(group(
                Some("Language"),
                vec![row(
                    "Preferred language",
                    Some("Your native language and the default translation target."),
                    select_box(&self.preferred_lang),
                    cx,
                )],
                cx,
            ))
            .child(group(
                Some("Shortcut"),
                {
                    let mut rows = vec![row(
                        format!("{shortcut} opens"),
                        Some("Where selected text goes when you press the shortcut."),
                        select_box(&self.shortcut_window),
                        cx,
                    )];
                    if cfg!(target_os = "macos") {
                        rows.push(row(
                            "Selection button",
                            Some("Show a button next to text you select with the mouse."),
                            Switch::new("selection-icon")
                                .checked(self.draft.selection_icon)
                                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                    this.draft.selection_icon = *checked;
                                    this.save(cx);
                                })),
                            cx,
                        ));
                        rows.push(row(
                            "Accessibility access",
                            Some("Needed to read the text selected in other apps."),
                            access_control,
                            cx,
                        ));
                    }
                    rows
                },
                cx,
            ))
            .child(
                div()
                    .w_full()
                    .text_center()
                    .child(caption(format!("Refiner {}", env!("CARGO_PKG_VERSION")), cx)),
            )
    }

    // ── Model ──────────────────────────────────────────────────────────────

    fn render_model(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let info = provider_info(&self.draft.provider);
        let suggested = format!("Suggested: {}", info.models.join(", "));
        let mut rows = vec![
            row("Provider", None, select_box(&self.provider), cx),
            row(
                "Model",
                Some(suggested.as_str()),
                field_box(Input::new(&self.model).small()),
                cx,
            ),
            row(
                "Server URL",
                Some("Leave empty to use the provider's default."),
                field_box(Input::new(&self.base_url).small()),
                cx,
            ),
        ];
        if info.needs_api_key {
            rows.push(row(
                "API key",
                None,
                field_box(Input::new(&self.api_key).small().mask_toggle()),
                cx,
            ));
        }

        v_flex()
            .gap_5()
            .child(group(Some("Provider"), rows, cx))
            .child(group(
                Some("Inference"),
                vec![row(
                    "Thinking",
                    Some("Let reasoning models think before answering. Slower, sometimes better."),
                    Switch::new("thinking")
                        .checked(self.draft.thinking)
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.draft.thinking = *checked;
                            this.save(cx);
                        })),
                    cx,
                )],
                cx,
            ))
    }

    // ── Prompts ────────────────────────────────────────────────────────────

    fn render_prompts(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_flex()
            .gap_5()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(
                        "Customize the instruction sent for each mode. Use {original_lang} and \
                         {target_lang} as placeholders. Leave a field empty to use the default.",
                    ),
            )
            .children(Mode::ALL.into_iter().zip(self.prompts.clone()).map(|(mode, state)| {
                let is_custom = !state.read(cx).value().trim().is_empty();
                let reset_state = state.clone();
                v_flex()
                    .gap_1p5()
                    .child(
                        h_flex()
                            .px_1()
                            .child(group_title(mode.label(), cx))
                            .child(div().flex_1())
                            .when(is_custom, |this| {
                                this.child(
                                    Button::new(SharedString::from(format!("reset-{}", mode.key())))
                                        .link()
                                        .xsmall()
                                        .label("Use Default")
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            reset_state
                                                .update(cx, |s, cx| s.set_value("", window, cx));
                                            this.save(cx);
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .rounded(px(10.))
                            .border_1()
                            .border_color(hairline(cx))
                            .bg(theme.muted.opacity(0.35))
                            .p_1()
                            .child(Textarea::new(&state).appearance(false).text_sm()),
                    )
                    .when(!is_custom, |this| {
                        this.child(
                            div()
                                .px_1()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("Default: {}", mode.default_prompt())),
                        )
                    })
            }))
    }

    // ── Analysis ───────────────────────────────────────────────────────────

    fn render_analysis(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let a = self.analysis.read(cx);
        let theme = cx.theme();
        let count = a.history_count;
        let running = a.is_running();
        let enabled = AppState::get(cx).settings.read(cx).history_enabled;

        let mut history_rows = vec![row(
            "Keep history",
            Some("Record each request so analysis has writing samples to work with."),
            Switch::new("history")
                .checked(enabled)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.set_history_enabled(*checked, cx)
                })),
            cx,
        )];
        if count > 0 {
            history_rows.push(row(
                format!("{count} {}", if count == 1 { "entry" } else { "entries" }),
                None,
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("export-history")
                            .small()
                            .label("Export…")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.analysis.update(cx, |a, cx| a.export(cx))
                            })),
                    )
                    .child(
                        Button::new("clear-history")
                            .small()
                            .when(self.confirm_clear, |b| b.danger())
                            .label(if self.confirm_clear { "Confirm Clear" } else { "Clear…" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.confirm_clear {
                                    this.confirm_clear = false;
                                    this.analysis.update(cx, |a, cx| a.clear(cx));
                                } else {
                                    this.confirm_clear = true;
                                }
                                cx.notify();
                            })),
                    ),
                cx,
            ));
        }

        let status: Option<AnyElement> = match &a.phase {
            Phase::Idle => None,
            Phase::Running { message, percent } => Some(
                v_flex()
                    .gap_2()
                    .px_3()
                    .py_2p5()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().flex_1().child(caption(message.clone(), cx)))
                            .child(caption(format!("{percent}%"), cx)),
                    )
                    .child(Progress::new("analysis-progress").value(*percent as f32))
                    .into_any_element(),
            ),
            Phase::Complete { report } => {
                let report = report.clone();
                Some(row(
                    "Report ready",
                    None,
                    Button::new("open-report")
                        .small()
                        .label("Open Report")
                        .on_click(move |_, _, _| history::open_path(&report)),
                    cx,
                ))
            }
            Phase::Error(err) => Some(
                div()
                    .px_3()
                    .py_2p5()
                    .text_sm()
                    .text_color(theme.danger)
                    .child(err.clone())
                    .into_any_element(),
            ),
        };

        let mut report_rows = vec![row(
            "Period",
            None,
            div().w(px(160.)).child(Select::new(&self.day_range).small().disabled(running)),
            cx,
        )];
        report_rows.extend(status);
        report_rows.push(
            h_flex()
                .px_3()
                .py_2()
                .gap_2()
                .justify_end()
                .child(
                    Button::new("open-reports-folder")
                        .small()
                        .ghost()
                        .icon(IconName::FolderOpen)
                        .label(if cfg!(target_os = "macos") {
                            "Show in Finder"
                        } else if cfg!(target_os = "windows") {
                            "Show in Explorer"
                        } else {
                            "Open Folder"
                        })
                        .on_click(|_, _, _| history::open_path(&data_dir())),
                )
                .child(
                    Button::new("run-analysis")
                        .primary()
                        .small()
                        .label(if running { "Analyzing…" } else { "Analyze Writing" })
                        .loading(running)
                        .disabled(count == 0 || running)
                        .on_click(cx.listener(|this, _, _, cx| {
                            let days = this.selected_days(cx);
                            this.analysis.update(cx, |a, cx| a.run(days, cx));
                        })),
                )
                .into_any_element(),
        );

        let reports = a.reports.clone();
        let feedback = a.feedback.clone();
        v_flex()
            .gap_5()
            .child(group(Some("History"), history_rows, cx))
            .when_some(feedback, |this, (is_error, msg)| {
                this.child(
                    div()
                        .px_1()
                        .text_sm()
                        .text_color(if is_error { theme.danger } else { theme.success })
                        .child(msg),
                )
            })
            .child(group(Some("Language report"), report_rows, cx))
            .when(!reports.is_empty(), |this| {
                this.child(group(
                    Some("Past reports"),
                    reports
                        .into_iter()
                        .enumerate()
                        .map(|(ix, r)| {
                            let path = r.path.clone();
                            row(
                                format_report_name(&r.filename),
                                None,
                                Button::new(("open-report-row", ix))
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::ExternalLink)
                                    .tooltip("Open")
                                    .on_click(move |_, _, _| history::open_path(&path)),
                                cx,
                            )
                        })
                        .collect(),
                    cx,
                ))
            })
    }
}

// ── Form building blocks ─────────────────────────────────────────────────────

fn group_title(title: &str, cx: &App) -> Div {
    div()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().foreground.opacity(0.85))
        .child(title.to_string())
}

/// A titled, rounded group of rows separated by hairlines.
fn group(title: Option<&str>, rows: Vec<AnyElement>, cx: &App) -> Div {
    let theme = cx.theme();
    let line = hairline(cx);
    v_flex()
        .gap_1p5()
        .when_some(title, |this, t| this.child(div().px_1().child(group_title(t, cx))))
        .child(
            v_flex()
                .rounded(px(10.))
                .border_1()
                .border_color(line)
                .bg(theme.muted.opacity(0.35))
                .overflow_hidden()
                .children(rows.into_iter().enumerate().map(move |(ix, r)| {
                    div()
                        .when(ix > 0, |this| this.border_t_1().border_color(line))
                        .child(r)
                })),
        )
}

/// Label (and optional description) on the left, control on the right.
fn row(
    label: impl Into<SharedString>,
    description: Option<&str>,
    control: impl IntoElement,
    cx: &App,
) -> AnyElement {
    let theme = cx.theme();
    h_flex()
        .min_h(px(40.))
        .px_3()
        .py_1p5()
        .gap_4()
        .justify_between()
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(div().text_sm().child(label.into()))
                .when_some(description, |this, d| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(d.to_string()),
                    )
                }),
        )
        .child(div().flex_shrink_0().child(control))
        .into_any_element()
}

fn select_box(state: &Entity<LabelSelect>) -> impl IntoElement {
    div().w(px(170.)).child(Select::new(state).small())
}

fn field_box(input: impl IntoElement) -> impl IntoElement {
    div().w(px(CONTROL_WIDTH)).child(input)
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content: AnyElement = match self.tab {
            SettingsTab::General => self.render_general(cx).into_any_element(),
            SettingsTab::Model => self.render_model(cx).into_any_element(),
            SettingsTab::Prompts => self.render_prompts(cx).into_any_element(),
            SettingsTab::Analysis => self.render_analysis(cx).into_any_element(),
        };
        let theme = cx.theme();
        v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.render_toolbar(cx))
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(div().w_full().max_w(px(560.)).mx_auto().px_6().py_5().child(content)),
            )
    }
}
