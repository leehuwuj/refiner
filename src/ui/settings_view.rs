//! Settings window: provider and app preferences (General) and history /
//! language analysis (Analysis).

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState, Textarea, TextareaState};
use gpui_kit::component::progress::Progress;
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, IndexPath, Sizable as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::track_appearance;
use crate::analysis::{Phase, format_report_name};
use crate::history;
use crate::refine::Mode;
use crate::settings::{
    LANGUAGES, PROVIDERS, Settings, ShortcutWindow, TextSize, ThemePref, data_dir, provider_info,
};
use crate::state::{AnalysisModel, AppState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Analysis,
}

const DAY_RANGES: [(Option<u32>, &str); 5] = [
    (Some(7), "Last 7 days"),
    (Some(14), "Last 14 days"),
    (Some(30), "Last 30 days"),
    (Some(90), "Last 90 days"),
    (None, "All time"),
];

type LabelSelect = SelectState<Vec<SharedString>>;

#[derive(Clone, Copy, PartialEq)]
enum SaveStatus {
    Idle,
    Saved,
    Failed,
}

pub struct SettingsView {
    tab: SettingsTab,
    /// Unsaved choices made with selects and switches.
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
    save_status: SaveStatus,
    confirm_clear: bool,
    _status_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

fn labels(items: impl IntoIterator<Item = &'static str>) -> Vec<SharedString> {
    items.into_iter().map(SharedString::from).collect()
}

fn row_index(ix: Option<usize>) -> Option<IndexPath> {
    ix.map(|ix| IndexPath::default().row(ix))
}

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
                .placeholder("Enter model name")
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
                .placeholder("sk-…")
                .default_value(s.api_key.clone())
        });
        let prompt = |mode: Mode, value: &str, window: &mut Window, cx: &mut Context<Self>| {
            let value = value.to_string();
            cx.new(|cx| {
                TextareaState::new(window, cx)
                    .rows(3)
                    .placeholder(mode.default_prompt())
                    .default_value(value)
            })
        };
        let prompts = [
            prompt(Mode::Translate, &s.prompt_translate, window, cx),
            prompt(Mode::Correct, &s.prompt_correct, window, cx),
            prompt(Mode::Refine, &s.prompt_refine, window, cx),
        ];

        let mut subs = vec![
            cx.subscribe_in(&provider, window, |this, _, e: &SelectEvent<Vec<SharedString>>, window, cx| {
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
                    cx.notify();
                }
            }),
            cx.subscribe_in(&shortcut_window, window, |this, _, e: &SelectEvent<Vec<SharedString>>, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                if let Some(w) = ShortcutWindow::ALL.into_iter().find(|w| w.label() == label.as_ref()) {
                    this.draft.shortcut_window = w;
                    cx.notify();
                }
            }),
            cx.subscribe_in(&preferred_lang, window, |this, _, e: &SelectEvent<Vec<SharedString>>, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                this.draft.preferred_lang = label.to_string();
                cx.notify();
            }),
            cx.subscribe_in(&theme, window, |this, _, e: &SelectEvent<Vec<SharedString>>, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                if let Some(t) = ThemePref::ALL.into_iter().find(|t| t.label() == label.as_ref()) {
                    this.draft.theme = t;
                    cx.notify();
                }
            }),
            cx.subscribe_in(&text_size, window, |this, _, e: &SelectEvent<Vec<SharedString>>, _, cx| {
                let SelectEvent::Confirm(Some(label)) = e else { return };
                if let Some(t) = TextSize::ALL.into_iter().find(|t| t.label() == label.as_ref()) {
                    this.draft.text_size = t;
                    cx.notify();
                }
            }),
            cx.observe(&state.analysis, |_, _, cx| cx.notify()),
        ];
        subs.extend(track_appearance(window, cx));

        state.analysis.update(cx, |a, cx| a.refresh(cx));

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
            save_status: SaveStatus::Idle,
            confirm_clear: false,
            _status_task: None,
            _subscriptions: subs,
        }
    }

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        self.tab = tab;
        if tab == SettingsTab::Analysis {
            self.analysis.update(cx, |a, cx| a.refresh(cx));
        }
        cx.notify();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let mut next = self.draft.clone();
        next.model = self.model.read(cx).value().trim().to_string();
        next.model_url = self.base_url.read(cx).value().trim().to_string();
        let key = self.api_key.read(cx).value().trim().to_string();
        if !key.is_empty() {
            next.api_key = key;
        }
        let [t, c, r] = &self.prompts;
        next.prompt_translate = t.read(cx).value().trim().to_string();
        next.prompt_correct = c.read(cx).value().trim().to_string();
        next.prompt_refine = r.read(cx).value().trim().to_string();
        // History is toggled live from the Analysis tab.
        next.history_enabled = AppState::get(cx).settings.read(cx).history_enabled;

        self.save_status = match AppState::save_settings(next.clone(), cx) {
            Ok(()) => {
                self.draft = next;
                SaveStatus::Saved
            }
            Err(err) => {
                eprintln!("refiner: failed to save settings: {err:#}");
                SaveStatus::Failed
            }
        };
        self._status_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(3))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.save_status = SaveStatus::Idle;
                cx.notify();
            });
        }));
        cx.notify();
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

    // ── General ────────────────────────────────────────────────────────────

    fn render_general(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let info = provider_info(&self.draft.provider);
        let theme = cx.theme();
        let suggestions = format!("Suggestions: {}", info.models.join(", "));
        let prompt_fields = Mode::ALL.iter().zip(&self.prompts).map(|(mode, state)| {
            v_flex()
                .gap_1p5()
                .child(div().text_sm().child(mode.label()))
                .child(Textarea::new(state).text_xs())
        });

        let save_label = match self.save_status {
            SaveStatus::Idle => "Save Settings",
            SaveStatus::Saved => "Saved",
            SaveStatus::Failed => "Save failed",
        };

        v_flex()
            .gap_4()
            .child(section(
                IconName::Bot,
                "AI Provider",
                body()
                    .child(row("Provider", None, cx).child(select_box(&self.provider)))
                    .child(field("Model", Some(suggestions), cx).child(Input::new(&self.model).small())),
                cx,
            ))
            .child(section(
                IconName::Key,
                "Authentication",
                body()
                    .child(field("Base URL", None, cx).child(Input::new(&self.base_url).small()))
                    .when(info.needs_api_key, |this| {
                        this.child(
                            field("API Key", None, cx)
                                .child(Input::new(&self.api_key).small().mask_toggle()),
                        )
                    }),
                cx,
            ))
            .child(section(
                IconName::Zap,
                "Inference",
                body().child(
                    row("Enable Thinking", Some("Extended reasoning for models that support it"), cx)
                        .child(
                            Switch::new("thinking")
                                .checked(self.draft.thinking)
                                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                    this.draft.thinking = *checked;
                                    cx.notify();
                                })),
                        ),
                ),
                cx,
            ))
            .child(section(
                IconName::Monitor,
                "Shortcut Behavior",
                body()
                    .child(
                        row("Window Type", Some(shortcut_description()), cx)
                            .child(select_box(&self.shortcut_window)),
                    )
                    .when(cfg!(target_os = "macos"), |this| {
                        this.child(
                            row(
                                "Selection Icon",
                                Some("Show a button next to text you select with the mouse"),
                                cx,
                            )
                            .child(
                                Switch::new("selection-icon")
                                    .checked(self.draft.selection_icon)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.draft.selection_icon = *checked;
                                        cx.notify();
                                    })),
                            ),
                        )
                    }),
                cx,
            ))
            .child(section(
                IconName::Languages,
                "Language",
                body().child(
                    row(
                        "Preferred Language",
                        Some("Your native language — also the default translation target"),
                        cx,
                    )
                    .child(select_box(&self.preferred_lang)),
                ),
                cx,
            ))
            .child(section(
                IconName::MessageSquare,
                "Prompts",
                body()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Variables: {original_lang}, {target_lang}. Leave blank for defaults."),
                    )
                    .children(prompt_fields),
                cx,
            ))
            .child(section(
                IconName::Palette,
                "Appearance",
                body()
                    .child(row("Theme", None, cx).child(select_box(&self.theme)))
                    .child(
                        row("Text Size", Some("Scales all text across windows"), cx)
                            .child(select_box(&self.text_size)),
                    ),
                cx,
            ))
            .child(
                Button::new("save-settings")
                    .primary()
                    .w_full()
                    .icon(if self.save_status == SaveStatus::Saved {
                        IconName::Check
                    } else {
                        IconName::Save
                    })
                    .label(save_label)
                    .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
            )
            .child(
                div()
                    .w_full()
                    .text_center()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .pb_2()
                    .child(format!("Refiner {} · GPUI", env!("CARGO_PKG_VERSION"))),
            )
    }

    // ── Analysis ───────────────────────────────────────────────────────────

    fn render_analysis(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let a = self.analysis.read(cx);
        let theme = cx.theme();
        let count = a.history_count;
        let running = a.is_running();
        let can_run = count > 0 && !running;
        let enabled = AppState::get(cx).settings.read(cx).history_enabled;

        let history_body = body()
            .child(
                row(
                    "Store input history",
                    Some("Record every AI request so language analysis has samples to work with"),
                    cx,
                )
                .child(
                    Switch::new("history")
                        .checked(enabled)
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.set_history_enabled(*checked, cx)
                        })),
                ),
            )
            .when(count > 0, |this| {
                this.child(
                    h_flex()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!(
                                    "{count} {} collected",
                                    if count == 1 { "entry" } else { "entries" }
                                )),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("export-history")
                                        .outline()
                                        .xsmall()
                                        .icon(IconName::Download)
                                        .label("Export JSON")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.analysis.update(cx, |a, cx| a.export(cx))
                                        })),
                                )
                                .child(
                                    Button::new("clear-history")
                                        .danger()
                                        .xsmall()
                                        .icon(IconName::Trash)
                                        .label(if self.confirm_clear {
                                            format!("Delete {count} entries?")
                                        } else {
                                            "Clear".to_string()
                                        })
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
                        ),
                )
            })
            .when_some(a.feedback.clone(), |this, (is_error, msg)| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(if is_error { theme.danger } else { theme.success })
                        .child(msg),
                )
            });
        let history_section = section(IconName::ChartColumn, "History Collection", history_body, cx);

        let status: Option<AnyElement> = match &a.phase {
            Phase::Idle => None,
            Phase::Running { message, percent } => Some(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .text_xs()
                            .child(
                                div()
                                    .flex_1()
                                    .text_color(theme.muted_foreground)
                                    .child(message.clone()),
                            )
                            .child(
                                div()
                                    .text_color(theme.primary)
                                    .child(format!("{percent}%")),
                            ),
                    )
                    .child(Progress::new("analysis-progress").value(*percent as f32))
                    .into_any_element(),
            ),
            Phase::Complete { report } => {
                let report = report.clone();
                Some(
                    h_flex()
                        .gap_2()
                        .child(
                            Icon::new(IconName::Check)
                                .small()
                                .text_color(theme.success),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_xs()
                                .text_color(theme.success)
                                .child("Analysis complete!"),
                        )
                        .child(
                            Button::new("open-report")
                                .primary()
                                .xsmall()
                                .icon(IconName::ExternalLink)
                                .label("Open Report")
                                .on_click(move |_, _, _| history::open_path(&report)),
                        )
                        .into_any_element(),
                )
            }
            Phase::Error(err) => Some(
                div()
                    .text_xs()
                    .text_color(theme.danger)
                    .child(err.clone())
                    .into_any_element(),
            ),
        };

        let run_label = if running {
            "Running…"
        } else if matches!(a.phase, Phase::Complete { .. }) {
            "Run Again"
        } else {
            "Run Analysis"
        };

        let run_body = body()
            .child(
                row("Date range", Some("Only analyse entries from this period"), cx)
                    .child(div().w(px(150.)).child(
                        Select::new(&self.day_range).small().disabled(running),
                    )),
            )
            .children(status)
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("run-analysis")
                            .primary()
                            .flex_1()
                            .small()
                            .icon(IconName::ChartColumn)
                            .label(run_label)
                            .loading(running)
                            .disabled(!can_run)
                            .tooltip(if count == 0 {
                                "Enable history and use the app first"
                            } else {
                                "Analyse your writing with the configured model"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                let days = this.selected_days(cx);
                                this.analysis.update(cx, |a, cx| a.run(days, cx));
                            })),
                    )
                    .child(
                        Button::new("open-reports-folder")
                            .outline()
                            .small()
                            .icon(IconName::FolderOpen)
                            .tooltip("Open reports folder")
                            .on_click(|_, _, _| history::open_path(&data_dir())),
                    ),
            );
        let run_section = section(IconName::RefreshCw, "Run Analysis", run_body, cx);

        let reports = a.reports.clone();
        v_flex()
            .gap_4()
            .child(history_section)
            .child(run_section)
            .when(!reports.is_empty(), |this| {
                this.child(
                    section(IconName::FileText, "Past Reports", body().gap_1().children(
                        reports.into_iter().enumerate().map(|(ix, r)| {
                            let path = r.path.clone();
                            h_flex()
                                .gap_2()
                                .px_2()
                                .py_1()
                                .rounded(px(8.))
                                .bg(theme.secondary.opacity(0.6))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .truncate()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(format_report_name(&r.filename)),
                                )
                                .child(
                                    Button::new(("open-report-row", ix))
                                        .ghost()
                                        .xsmall()
                                        .icon(IconName::ExternalLink)
                                        .tooltip("Open")
                                        .on_click(move |_, _, _| history::open_path(&path)),
                                )
                        }),
                    ), cx),
                )
            })
    }
}

fn url_placeholder(provider: &str) -> &'static str {
    if provider == "ollama" {
        "http://localhost:11434"
    } else {
        "Leave empty to use default"
    }
}

fn shortcut_description() -> &'static str {
    if cfg!(target_os = "macos") {
        "Which window opens on ⌘E"
    } else {
        "Which window opens on Ctrl+E"
    }
}

fn section(icon: IconName, title: &'static str, body: Div, cx: &App) -> Div {
    let theme = cx.theme();
    v_flex()
        .rounded(px(12.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.muted.opacity(0.3))
        .overflow_hidden()
        .child(
            h_flex()
                .gap_2()
                .px_4()
                .py_2p5()
                .border_b_1()
                .border_color(theme.border)
                .child(Icon::new(icon).small().text_color(theme.primary))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.muted_foreground)
                        .child(title.to_uppercase()),
                ),
        )
        .child(body.px_4().py_3())
}

/// The padded body of a section.
fn body() -> Div {
    v_flex().gap_3()
}

/// A label (with optional description) on the left and a control on the right.
fn row(label: &'static str, description: Option<&'static str>, cx: &App) -> Div {
    let theme = cx.theme();
    h_flex().gap_4().justify_between().child(
        v_flex()
            .flex_1()
            .min_w_0()
            .child(div().text_sm().child(label))
            .when_some(description, |this, d| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(d),
                )
            }),
    )
}

/// A stacked label (with optional description) above a full-width control.
fn field(label: &'static str, description: Option<String>, cx: &App) -> Div {
    let theme = cx.theme();
    v_flex()
        .gap_1p5()
        .child(div().text_sm().child(label))
        .when_some(description, |this, d| {
            this.child(div().text_xs().text_color(theme.muted_foreground).child(d))
        })
}

fn select_box(state: &Entity<LabelSelect>) -> impl IntoElement {
    div().w(px(160.)).child(Select::new(state).small())
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = match self.tab {
            SettingsTab::General => 0,
            SettingsTab::Analysis => 1,
        };
        let content: AnyElement = match self.tab {
            SettingsTab::General => self.render_general(cx).into_any_element(),
            SettingsTab::Analysis => self.render_analysis(cx).into_any_element(),
        };
        let theme = cx.theme();
        v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                h_flex()
                    .px_4()
                    .py_2p5()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        TabBar::new("settings-tabs")
                            .segmented()
                            .small()
                            .selected_index(selected)
                            .child(Tab::new().label("General"))
                            .child(Tab::new().label("Analysis"))
                            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                let tab = if *ix == 0 {
                                    SettingsTab::General
                                } else {
                                    SettingsTab::Analysis
                                };
                                this.set_tab(tab, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_4()
                    .child(content),
            )
    }
}
