//! Application models shared across windows.
//!
//! Windows are closed when they lose focus and reopened on demand, so all
//! state that must survive that (input text, results, in-flight requests)
//! lives here rather than in the views.

use std::path::PathBuf;

use gpui_kit::*;

use crate::analysis::{self, Phase, ReportInfo};
use crate::history;
use crate::refine::{self, Mode, Request};
use crate::settings::Settings;

/// Outcome of one request: the text, or an error message.
pub type Outcome = Result<String, String>;

/// Handles to every model, stored as a GPUI global.
#[derive(Clone)]
pub struct AppState {
    pub settings: Entity<Settings>,
    pub main: Entity<MainModel>,
    pub popup: Entity<PopupModel>,
    pub analysis: Entity<AnalysisModel>,
}

impl Global for AppState {}

impl AppState {
    pub fn init(cx: &mut App) {
        let settings = Settings::load();
        let target = settings.preferred_lang.clone();
        let state = AppState {
            settings: cx.new(|_| settings),
            main: cx.new(|_| MainModel::new(target)),
            popup: cx.new(|_| PopupModel::default()),
            analysis: cx.new(|_| AnalysisModel::new()),
        };
        cx.set_global(state);
    }

    pub fn get(cx: &App) -> &AppState {
        cx.global::<AppState>()
    }

    pub fn settings(cx: &App) -> Settings {
        Self::get(cx).settings.read(cx).clone()
    }

    /// Replaces and persists the settings.
    pub fn save_settings(new: Settings, cx: &mut App) -> anyhow::Result<()> {
        new.save()?;
        let state = Self::get(cx).clone();
        let preferred_changed = state.settings.read(cx).preferred_lang != new.preferred_lang;
        let preferred = new.preferred_lang.clone();
        state.settings.update(cx, |s, cx| {
            *s = new;
            cx.notify();
        });
        if preferred_changed {
            state.main.update(cx, |m, cx| {
                m.target_lang = preferred;
                cx.notify();
            });
        }
        Ok(())
    }
}

// ── Main window ─────────────────────────────────────────────────────────────

pub struct MainModel {
    pub input: String,
    pub mode: Mode,
    pub source_lang: String,
    pub target_lang: String,
    results: [Option<Outcome>; 3],
    pub loading: bool,
    task: Option<Task<()>>,
}

impl MainModel {
    fn new(target_lang: String) -> Self {
        Self {
            input: String::new(),
            mode: Mode::Translate,
            source_lang: "English".into(),
            target_lang,
            results: [None, None, None],
            loading: false,
            task: None,
        }
    }

    pub fn result(&self) -> Option<&Outcome> {
        self.results[self.mode.index()].as_ref()
    }

    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if self.input != text {
            self.input = text;
            cx.notify();
        }
    }

    pub fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if self.mode != mode {
            self.mode = mode;
            if !self.input.trim().is_empty() {
                self.run(cx);
            }
            cx.notify();
        }
    }

    pub fn swap_languages(&mut self, cx: &mut Context<Self>) {
        std::mem::swap(&mut self.source_lang, &mut self.target_lang);
        cx.notify();
    }

    pub fn set_target_lang(&mut self, lang: String, cx: &mut Context<Self>) {
        self.target_lang = lang;
        cx.notify();
    }

    /// Runs the current mode on the current input.
    pub fn run(&mut self, cx: &mut Context<Self>) {
        if self.input.trim().is_empty() {
            return;
        }
        let settings = AppState::settings(cx);
        let mode = self.mode;
        let req = Request {
            mode,
            text: self.input.clone(),
            source_lang: self.source_lang.clone(),
            target_lang: self.target_lang.clone(),
        };
        self.results[mode.index()] = None;
        self.loading = true;
        cx.notify();

        let job = cx
            .background_executor()
            .spawn(async move { refine::run(&settings, &req) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let outcome = job.await;
            let _ = this.update(cx, |m, cx| {
                m.results[mode.index()] = Some(outcome);
                m.loading = false;
                m.task = None;
                cx.notify();
            });
        }));
    }

    /// Shows a canned result (window snapshots).
    #[cfg(feature = "snapshot")]
    pub fn demo(&mut self, input: &str, output: &str, cx: &mut Context<Self>) {
        self.input = input.to_string();
        self.results[self.mode.index()] = Some(Ok(output.to_string()));
        cx.notify();
    }

    /// Drops the in-flight request; its result is discarded.
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.loading = false;
        cx.notify();
    }
}

// ── Quick popup ─────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct PopupModel {
    pub translation: Option<Outcome>,
    pub correction: Option<Outcome>,
    tasks: Vec<Task<()>>,
}

impl PopupModel {
    pub fn translate_loading(&self) -> bool {
        self.translation.is_none() && !self.tasks.is_empty()
    }

    pub fn correct_loading(&self) -> bool {
        self.correction.is_none() && !self.tasks.is_empty()
    }

    #[cfg(feature = "snapshot")]
    pub fn demo(&mut self, translation: &str, correction: &str, cx: &mut Context<Self>) {
        self.translation = Some(Ok(translation.to_string()));
        self.correction = Some(Ok(correction.to_string()));
        cx.notify();
    }

    /// Translates the text into the preferred language and corrects it in its
    /// own language, in parallel.
    pub fn run(&mut self, text: String, cx: &mut Context<Self>) {
        let settings = AppState::settings(cx);
        self.translation = None;
        self.correction = None;
        self.tasks.clear();

        let translate = Request {
            mode: Mode::Translate,
            text: text.clone(),
            source_lang: "auto".into(),
            target_lang: settings.preferred_lang.clone(),
        };
        let correct = Request {
            mode: Mode::Correct,
            text,
            source_lang: "auto".into(),
            target_lang: "the original language of the text".into(),
        };
        for (req, is_translation) in [(translate, true), (correct, false)] {
            let settings = settings.clone();
            let job = cx
                .background_executor()
                .spawn(async move { refine::run(&settings, &req) });
            self.tasks.push(cx.spawn(async move |this, cx| {
                let outcome = job.await;
                let _ = this.update(cx, |m, cx| {
                    if is_translation {
                        m.translation = Some(outcome);
                    } else {
                        m.correction = Some(outcome);
                    }
                    if m.translation.is_some() && m.correction.is_some() {
                        m.tasks.clear();
                    }
                    cx.notify();
                });
            }));
        }
        cx.notify();
    }
}

// ── Language analysis ───────────────────────────────────────────────────────

pub struct AnalysisModel {
    pub phase: Phase,
    pub history_count: usize,
    pub reports: Vec<ReportInfo>,
    /// Short-lived feedback line: (is_error, message).
    pub feedback: Option<(bool, String)>,
    task: Option<Task<()>>,
    feedback_task: Option<Task<()>>,
}

impl AnalysisModel {
    fn new() -> Self {
        Self {
            phase: Phase::Idle,
            history_count: history::count(),
            reports: analysis::list_reports(),
            feedback: None,
            task: None,
            feedback_task: None,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.phase, Phase::Running { .. })
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.history_count = history::count();
        self.reports = analysis::list_reports();
        cx.notify();
    }

    pub fn flash(&mut self, is_error: bool, message: impl Into<String>, cx: &mut Context<Self>) {
        self.feedback = Some((is_error, message.into()));
        self.feedback_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(3500))
                .await;
            let _ = this.update(cx, |m, cx| {
                m.feedback = None;
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub fn export(&mut self, cx: &mut Context<Self>) {
        match history::export_json() {
            Ok(path) => {
                history::open_path(&path);
                self.flash(false, "Exported — opened in default app.", cx);
            }
            Err(e) => self.flash(true, e, cx),
        }
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        match history::clear() {
            Ok(()) => {
                self.history_count = 0;
                self.flash(false, "History cleared.", cx);
            }
            Err(e) => self.flash(true, e, cx),
        }
    }

    pub fn run(&mut self, days_back: Option<u32>, cx: &mut Context<Self>) {
        if self.is_running() {
            return;
        }
        let settings = AppState::settings(cx);
        self.phase = Phase::Running {
            message: "Starting analysis…".into(),
            percent: 0,
        };
        cx.notify();

        let (tx, rx) = smol::channel::unbounded::<(String, u8)>();
        let job = cx.background_executor().spawn(async move {
            analysis::run(&settings, days_back, |message, percent| {
                let _ = tx.try_send((message.to_string(), percent));
            })
        });
        self.task = Some(cx.spawn(async move |this, cx| {
            // Progress messages end when the job drops its sender.
            while let Ok((message, percent)) = rx.recv().await {
                let _ = this.update(cx, |m, cx| {
                    m.phase = Phase::Running { message, percent };
                    cx.notify();
                });
            }
            let result: Result<PathBuf, String> = job.await;
            let _ = this.update(cx, |m, cx| {
                m.phase = match result {
                    Ok(report) => {
                        history::open_path(&report);
                        Phase::Complete { report }
                    }
                    Err(e) => Phase::Error(e),
                };
                m.task = None;
                m.reports = analysis::list_reports();
                cx.notify();
            });
        }));
    }
}
