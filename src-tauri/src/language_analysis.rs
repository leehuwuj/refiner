use crate::history::{load_history_file, open_path};
use crate::providers::base::{get_provider, Provider};
use serde::Serialize;
use std::fs;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

// ── Date helpers ───────────────────────────────────────────────────────────────

/// Returns a filename-safe timestamp string: `analysis_YYYY-MM-DD_HH-MM.html`
fn timestamped_report_name() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let min = (rem % 3600) / 60;

    // Howard Hinnant civil-from-days algorithm
    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mon: i64 = if mp < 10 { mp as i64 + 3 } else { mp as i64 - 9 };
    let year = if mon <= 2 { y + 1 } else { y };

    format!("analysis_{:04}-{:02}-{:02}_{:02}-{:02}.html", year, mon, d, h, min)
}

// ── Report listing ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ReportInfo {
    pub filename: String,
    pub path: String,
}

// ── Shared analysis state (lets the window query status after mounting) ────────

#[derive(Default, Clone, Serialize)]
pub struct AnalysisStatus {
    pub running: bool,
    pub message: String,
    pub percent: u8,
    pub complete: bool,
    pub report_path: Option<String>,
    pub error: Option<String>,
}

pub struct AppAnalysisState(pub Arc<Mutex<AnalysisStatus>>);

// ── Event payloads ─────────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
struct AnalysisProgress {
    message: String,
    percent: u8,
}

#[derive(Clone, Serialize)]
struct AnalysisComplete {
    path: String,
}

#[derive(Clone, Serialize)]
struct AnalysisError {
    error: String,
}

// ── AI Prompt ──────────────────────────────────────────────────────────────────

const ANALYSIS_PROMPT: &str = r#"You are an expert linguist and language teacher specialising in second-language acquisition (SLA).

Below are writing samples from a learner collected across {ENTRY_COUNT} sessions using a translation and correction tool.

TARGET LANGUAGE FOR ANALYSIS: {TARGET_LANG}
Prioritise samples written in, or corrected to, this language.

--- SAMPLES ---
{FORMATTED_SAMPLES}
--- END SAMPLES ---

Produce a comprehensive, visually polished standalone HTML report assessing the learner's L2 proficiency. Requirements:
- All CSS MUST be embedded inside a single <style> tag — no external stylesheets, CDN links, or JavaScript libraries
- Support both light and dark mode using @media (prefers-color-scheme: dark)
- Clean, modern typography; max reading width 720px; comfortable line-height

Structure the report with these six sections:

1. EXECUTIVE SUMMARY
   - Estimated CEFR level (A1–C2) with a brief one-sentence justification
   - 2–3 sentence narrative of current level and learning trajectory

2. GRAMMAR & ACCURACY
   - Recurring error patterns (quote verbatim examples from the samples)
   - Grammar structures clearly mastered
   - Top 3–5 weakness areas, each as: [verbatim error] → [corrected form] → [plain-English rule]
   - Colour-code frequency: rose/red = high, amber = moderate, emerald = rare

3. VOCABULARY & RANGE
   - Vocabulary breadth: limited / developing / broad / advanced
   - Overused words or phrases with 2–3 suggested alternatives each
   - Register consistency (formal/informal mixing issues if any)
   - 5–8 vocabulary items to learn next, each with a short example sentence

4. FLUENCY & NATURALNESS
   - Sentence structure variety and syntactic complexity
   - L1 interference patterns — spots where native-language syntax bleeds through
   - Overall native-speaker naturalness assessment

5. PROGRESS TREND
   - Compare early vs recent samples: what has clearly improved?
   - What patterns persist despite repeated exposure?
   (If fewer than 10 samples exist, note that a trend analysis needs more data)

6. PERSONALISED STUDY PLAN
   - 3–5 specific, actionable weekly exercises matched to identified weaknesses
   - Most beneficial learning activity types for this learner's profile (e.g. shadowing, spaced repetition, dictation)

VISUAL DESIGN:
- Include a pure SVG skill radar chart (no JS) covering five axes: Grammar, Vocabulary, Fluency, Naturalness, Range — score each 1–10 based on your analysis
- SVG radar chart MUST use viewBox="0 0 500 500" with the pentagon centered at cx=250 cy=260. Place axis labels at least 30px beyond the outermost pentagon ring so they are never clipped. Use font-size="14" for labels and anchor them with text-anchor based on their position (start/middle/end). Wrap the SVG in a div with overflow:visible so nothing is cut off.
- Use a professional card-based layout with subtle shadows
- Footer: "Generated by Refiner • {ENTRY_COUNT} samples analysed"

Return ONLY the complete HTML document, starting exactly with <!DOCTYPE html>. No markdown fences, no explanation, no preamble."#;

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Returns only the entries that represent L2 writing: mode is correct or
/// refine, and target_lang differs from the user's preferred (native) language.
/// Falls back to all correct/refine entries if nothing matches.
fn l2_entries<'a>(
    entries: &'a [crate::history::HistoryEntry],
    native_lang: &str,
) -> Vec<&'a crate::history::HistoryEntry> {
    let correction_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.mode == "correct" || e.mode == "refine")
        .collect();

    // Prefer entries whose target lang is not the user's native language
    let l2: Vec<_> = correction_entries
        .iter()
        .copied()
        .filter(|e| !e.target_lang.eq_ignore_ascii_case(native_lang))
        .collect();

    if !l2.is_empty() {
        l2
    } else {
        correction_entries
    }
}

fn determine_l2_lang(
    entries: &[crate::history::HistoryEntry],
    native_lang: &str,
) -> String {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for e in l2_entries(entries, native_lang) {
        *counts.entry(e.target_lang.clone()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(lang, _)| lang)
        .unwrap_or_else(|| "English".to_string())
}

fn format_samples(
    entries: &[crate::history::HistoryEntry],
    native_lang: &str,
) -> String {
    l2_entries(entries, native_lang)
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let text = if e.input_text.len() > 1500 {
                format!("{}…", &e.input_text[..1500])
            } else {
                e.input_text.clone()
            };
            format!(
                "[{}] mode={} | {} → {}\n{}",
                i + 1,
                e.mode,
                e.source_lang,
                e.target_lang,
                text.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

fn extract_html(content: &str) -> String {
    if let Some(start) = content
        .find("<!DOCTYPE html>")
        .or_else(|| content.find("<!doctype html>"))
        .or_else(|| content.find("<html"))
    {
        content[start..].to_string()
    } else {
        content.to_string()
    }
}

fn set_status(app: &tauri::AppHandle, status: AnalysisStatus) {
    if let Some(state) = app.try_state::<AppAnalysisState>() {
        if let Ok(mut lock) = state.0.lock() {
            *lock = status.clone();
        }
    }
    // Mirror to events so the window stays in sync
    if status.complete {
        if let Some(path) = &status.report_path {
            let _ = app.emit("analysis-complete", AnalysisComplete { path: path.clone() });
        }
    } else if status.error.is_some() {
        let _ = app.emit(
            "analysis-error",
            AnalysisError {
                error: status.error.unwrap_or_default(),
            },
        );
    } else {
        let _ = app.emit(
            "analysis-progress",
            AnalysisProgress {
                message: status.message.clone(),
                percent: status.percent,
            },
        );
    }
}

// ── Commands ───────────────────────────────────────────────────────────────────

/// Returns the current analysis state — called by the window on mount so it
/// can catch up if it was opened after a run had already started.
#[tauri::command]
pub async fn get_analysis_status(app_handle: tauri::AppHandle) -> AnalysisStatus {
    app_handle
        .try_state::<AppAnalysisState>()
        .and_then(|s| s.0.lock().ok().map(|g| g.clone()))
        .unwrap_or_default()
}

/// Opens a specific report by full path.
#[tauri::command]
pub async fn open_report(path: String) -> Result<(), String> {
    if !std::path::Path::new(&path).exists() {
        return Err("Report file not found.".to_string());
    }
    open_path(&path);
    Ok(())
}

/// Opens the app-data directory (where all reports live) in Finder / Explorer.
#[tauri::command]
pub async fn open_reports_folder(app_handle: tauri::AppHandle) -> Result<(), String> {
    let dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let _ = fs::create_dir_all(&dir);
    open_path(dir.to_string_lossy().as_ref());
    Ok(())
}

/// Lists all saved analysis reports, newest first.
#[tauri::command]
pub async fn list_reports(app_handle: tauri::AppHandle) -> Result<Vec<ReportInfo>, String> {
    let dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut reports = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("analysis_") && name.ends_with(".html") {
                reports.push(ReportInfo {
                    filename: name,
                    path: entry.path().to_string_lossy().to_string(),
                });
            }
        }
    }
    reports.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(reports)
}

/// Opens the most recently saved report.
#[tauri::command]
pub async fn open_last_report(app_handle: tauri::AppHandle) -> Result<(), String> {
    let reports = list_reports(app_handle).await?;
    let latest = reports.into_iter().next()
        .ok_or_else(|| "No report found. Run analysis first.".to_string())?;
    open_path(&latest.path);
    Ok(())
}

/// Spawns the background analysis job.  Returns quickly; progress arrives via
/// `analysis-progress`, `analysis-complete`, and `analysis-error` events, and
/// is also queryable via `get_analysis_status`.
#[tauri::command]
pub async fn run_language_analysis(
    app_handle: tauri::AppHandle,
    days_back: Option<u32>,
) -> Result<(), String> {
    let history = load_history_file(&app_handle)?;
    if history.entries.is_empty() {
        return Err(
            "No history entries found. Enable history collection and use the app first."
                .to_string(),
        );
    }

    set_status(
        &app_handle,
        AnalysisStatus {
            running: true,
            message: "Starting analysis…".to_string(),
            percent: 0,
            ..Default::default()
        },
    );

    let app = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        run_analysis_inner(app, days_back).await;
    });

    Ok(())
}

async fn run_analysis_inner(app: tauri::AppHandle, days_back: Option<u32>) {
    // 1 – Load and optionally filter entries by date range
    set_status(
        &app,
        AnalysisStatus {
            running: true,
            message: "Loading history entries…".to_string(),
            percent: 5,
            ..Default::default()
        },
    );

    let history = match load_history_file(&app) {
        Ok(h) => h,
        Err(e) => {
            set_status(&app, AnalysisStatus { error: Some(e), ..Default::default() });
            return;
        }
    };

    let cutoff_ms = days_back.map(|d| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|t| t.as_millis() as u64)
            .unwrap_or(0);
        now.saturating_sub(d as u64 * 86_400 * 1_000)
    });

    let filtered: Vec<_> = history.entries.iter()
        .filter(|e| cutoff_ms.map_or(true, |c| e.timestamp >= c))
        .cloned()
        .collect();
    let entries = &filtered;

    // Resolve the user's native language from settings so we can isolate L2
    let native_lang = {
        match app.store("store.bin") {
            Ok(s) => s.get("PREFERRED_LANG")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "Vietnamese".to_string()),
            Err(_) => "Vietnamese".to_string(),
        }
    };

    let l2_count = l2_entries(entries, &native_lang).len();
    if l2_count == 0 {
        set_status(
            &app,
            AnalysisStatus {
                error: Some(format!(
                    "No correction or refinement entries found in your history. \
                     Use the Correct or Refine modes on your {} writing first, then run analysis again.",
                    native_lang
                )),
                ..Default::default()
            },
        );
        return;
    }

    let target_lang = determine_l2_lang(entries, &native_lang);
    let formatted = format_samples(entries, &native_lang);

    // 2 – Build prompt
    let prompt = ANALYSIS_PROMPT
        .replace("{ENTRY_COUNT}", &entries.len().to_string())
        .replace("{TARGET_LANG}", &target_lang)
        .replace("{FORMATTED_SAMPLES}", &formatted);

    // 3 – Resolve provider
    let store = match app.store("store.bin") {
        Ok(s) => s,
        Err(e) => {
            set_status(
                &app,
                AnalysisStatus {
                    error: Some(e.to_string()),
                    ..Default::default()
                },
            );
            return;
        }
    };

    let provider_name = store
        .get("PROVIDER")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "ollama".to_string());
    let model_name = store
        .get("MODEL")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "gemma3".to_string());

    set_status(
        &app,
        AnalysisStatus {
            running: true,
            message: format!(
                "Analysing {} {} correction samples with {} — this may take a minute…",
                l2_count,
                target_lang,
                model_name
            ),
            percent: 20,
            ..Default::default()
        },
    );

    let provider = get_provider(app.clone(), &provider_name, &model_name);

    let full_prompt = format!(
        "<start_of_turn>user\n{}\n<end_of_turn>\n<start_of_turn>model",
        prompt
    );

    // 4 – Call AI
    let result = provider.completion(&full_prompt).await;

    set_status(
        &app,
        AnalysisStatus {
            running: true,
            message: "Generating HTML report…".to_string(),
            percent: 85,
            ..Default::default()
        },
    );

    let html = match result {
        Ok(content) => {
            let cleaned = crate::commands::trim_thinking_blocks(&content);
            extract_html(&cleaned)
        }
        Err(e) => {
            set_status(
                &app,
                AnalysisStatus {
                    error: Some(format!("AI error: {}", e)),
                    ..Default::default()
                },
            );
            return;
        }
    };

    // 5 – Save HTML
    let dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            set_status(
                &app,
                AnalysisStatus {
                    error: Some(e.to_string()),
                    ..Default::default()
                },
            );
            return;
        }
    };
    let _ = fs::create_dir_all(&dir);
    let filename = timestamped_report_name();
    let report_path = dir.join(&filename);
    if let Err(e) = fs::write(&report_path, html) {
        set_status(&app, AnalysisStatus { error: Some(format!("Failed to save report: {}", e)), ..Default::default() });
        return;
    }

    let path_str = report_path.to_string_lossy().to_string();

    // 6 – Done
    set_status(
        &app,
        AnalysisStatus {
            complete: true,
            message: "Analysis complete!".to_string(),
            percent: 100,
            report_path: Some(path_str.clone()),
            ..Default::default()
        },
    );

    open_path(&path_str);
}
