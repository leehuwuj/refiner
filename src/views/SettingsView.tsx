import React, { useContext, useState, useEffect, useRef } from "react";
import {
  Save, Bot, Key, Zap, Monitor, Palette, Check, MessageSquare, Languages,
  BarChart3, Download, Trash2, ExternalLink, FolderOpen, RefreshCw,
  Loader2,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { SettingContext } from "@/providers/settings";
import { providerMap } from "@/types/settings";
import type { ShortcutWindowType, TextSizeType } from "@/types/settings";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { useTheme } from "@/providers/theme";
import type { ThemeType } from "@/types/settings";
import { cn } from "@/lib/utils";

// ── Constants ─────────────────────────────────────────────────────────────────

const LANGUAGES = [
  { value: "en", label: "English" },
  { value: "vi", label: "Tiếng Việt" },
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "ja", label: "日本語" },
  { value: "zh", label: "中文" },
  { value: "ko", label: "한국어" },
  { value: "es", label: "Español" },
];

const DEFAULT_PROMPTS = {
  translate:
    "You are an expert translator. Translate the following text from {original_lang} to {target_lang}. Provide only the translated text, without any additional explanations or formatting.",
  correct:
    "You are an expert in grammar. Correct the grammar of the following text in {target_lang}. Provide only the corrected text, without any additional explanations or formatting.",
  refine:
    "You are an expert editor. Rewrite the following text in a more conversational style, in {target_lang}. Output a single rewrite only — no options, no alternatives, no explanations, no labels, no formatting.",
};

const DAY_RANGE_OPTIONS = [
  { value: "7", label: "Last 7 days" },
  { value: "14", label: "Last 14 days" },
  { value: "30", label: "Last 30 days" },
  { value: "90", label: "Last 90 days" },
  { value: "0", label: "All time" },
];

// ── Shared sub-components ─────────────────────────────────────────────────────

function Section({
  icon: Icon, title, children,
}: {
  icon: React.ComponentType<{ size?: number; className?: string }>;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-xl border border-[var(--glass-border)] bg-[var(--glass-panel-bg)] overflow-hidden">
      <div className="flex items-center gap-2 px-4 py-3 border-b border-[var(--glass-border)]">
        <div className="p-1 rounded-md bg-[var(--accent-subtle)]">
          <Icon size={13} className="text-[var(--accent)]" />
        </div>
        <span className="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider">
          {title}
        </span>
      </div>
      <div className="px-4 py-3 space-y-3">{children}</div>
    </div>
  );
}

function Row({ label, description, children }: {
  label: string; description?: string; children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex-1 min-w-0">
        <p className="text-sm text-[var(--text-primary)]">{label}</p>
        {description && (
          <p className="text-[11px] text-[var(--text-tertiary)] mt-0.5">{description}</p>
        )}
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

// ── Analysis tab ──────────────────────────────────────────────────────────────

interface ReportInfo { filename: string; path: string; }

function formatReportName(filename: string): string {
  const m = filename.match(/analysis_(\d{4})-(\d{2})-(\d{2})_(\d{2})-(\d{2})\.html/);
  if (!m) return filename;
  const [, year, mon, day, h, min] = m;
  const date = new Date(+year, +mon - 1, +day);
  const monthStr = date.toLocaleDateString("en-US", { month: "short" });
  return `${monthStr} ${+day}, ${year} · ${h}:${min}`;
}

type AnalysisPhase = "idle" | "running" | "complete" | "error";

interface AnalysisState {
  phase: AnalysisPhase;
  message: string;
  percent: number;
  reportPath?: string;
  error?: string;
}

function AnalysisTab() {
  const [enabled, setEnabled] = useState(false);
  const [count, setCount] = useState(0);
  const [daysBack, setDaysBack] = useState("30");
  const [reports, setReports] = useState<ReportInfo[]>([]);
  const [analysis, setAnalysis] = useState<AnalysisState>({ phase: "idle", message: "", percent: 0 });
  const [feedback, setFeedback] = useState<{ type: "ok" | "err"; msg: string } | null>(null);
  const unlistenRef = useRef<Array<() => void>>([]);

  function flash(type: "ok" | "err", msg: string, ms = 3500) {
    setFeedback({ type, msg });
    setTimeout(() => setFeedback(null), ms);
  }

  function refreshReports() {
    invoke<ReportInfo[]>("list_reports").then(setReports).catch(() => {});
  }

  // Subscribe to analysis events emitted by the Rust backend
  useEffect(() => {
    // Catch up with any in-progress run that started before this tab mounted
    invoke<{
      running: boolean; message: string; percent: number;
      complete: boolean; report_path?: string; error?: string;
    }>("get_analysis_status").then((s) => {
      if (s.running) {
        setAnalysis({ phase: "running", message: s.message, percent: s.percent });
      } else if (s.complete && s.report_path) {
        setAnalysis({ phase: "complete", message: "Analysis complete!", percent: 100, reportPath: s.report_path });
        refreshReports();
      } else if (s.error) {
        setAnalysis({ phase: "error", message: "", percent: 0, error: s.error });
      }
    }).catch(() => {});

    const setup = async () => {
      const u1 = await listen<{ message: string; percent: number }>("analysis-progress", (e) => {
        setAnalysis({ phase: "running", message: e.payload.message, percent: e.payload.percent });
      });
      const u2 = await listen<{ path: string }>("analysis-complete", (e) => {
        setAnalysis({ phase: "complete", message: "Analysis complete!", percent: 100, reportPath: e.payload.path });
        refreshReports();
      });
      const u3 = await listen<{ error: string }>("analysis-error", (e) => {
        setAnalysis({ phase: "error", message: "", percent: 0, error: e.payload.error });
      });
      unlistenRef.current = [u1, u2, u3];
    };
    setup();

    return () => { unlistenRef.current.forEach((fn) => fn()); };
  }, []);

  useEffect(() => {
    invoke<boolean>("get_history_enabled").then(setEnabled).catch(() => {});
    invoke<number>("get_history_count").then(setCount).catch(() => {});
    refreshReports();
  }, []);

  async function handleToggle(val: boolean) {
    setEnabled(val);
    await invoke("toggle_history", { enabled: val }).catch(() => {});
  }

  async function handleExport() {
    try {
      await invoke("export_history_json");
      flash("ok", "Exported — opened in default app.");
    } catch (e) { flash("err", String(e)); }
  }

  async function handleClear() {
    if (!window.confirm(`Clear all ${count} history entries? This cannot be undone.`)) return;
    await invoke("clear_history").catch(() => {});
    setCount(0);
    flash("ok", "History cleared.");
  }

  async function handleRunAnalysis() {
    setAnalysis({ phase: "running", message: "Starting…", percent: 0 });
    try {
      const days = daysBack === "0" ? null : parseInt(daysBack);
      await invoke("run_language_analysis", { daysBack: days });
    } catch (e) {
      setAnalysis({ phase: "error", message: "", percent: 0, error: String(e) });
    }
  }

  async function handleOpenFolder() {
    try { await invoke("open_reports_folder"); }
    catch (e) { flash("err", String(e)); }
  }

  async function handleOpenReport(path: string) {
    try { await invoke("open_report", { path }); }
    catch (e) { flash("err", String(e)); }
  }

  const canRun = count > 0 && analysis.phase !== "running";

  return (
    <div className="space-y-4">
      {/* History collection */}
      <Section icon={BarChart3} title="History Collection">
        <Row
          label="Store input history"
          description="Record every AI request so language analysis has samples to work with"
        >
          <Switch checked={enabled} onCheckedChange={handleToggle} />
        </Row>
        {count > 0 && (
          <div className="flex items-center justify-between gap-2 pt-0.5">
            <span className="text-[11px] text-[var(--text-tertiary)]">
              {count} {count === 1 ? "entry" : "entries"} collected
            </span>
            <div className="flex gap-2">
              <button
                onClick={handleExport}
                className="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium"
                style={{ background: "var(--glass-control-bg)", border: "1px solid var(--chip-border)", color: "var(--text-secondary)", cursor: "pointer" }}
              >
                <Download size={10} /> Export JSON
              </button>
              <button
                onClick={handleClear}
                className="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium"
                style={{ background: "rgba(248,113,113,0.08)", border: "1px solid rgba(248,113,113,0.25)", color: "#f87171", cursor: "pointer" }}
              >
                <Trash2 size={10} /> Clear
              </button>
            </div>
          </div>
        )}
        <AnimatePresence>
          {feedback && (
            <motion.p
              initial={{ opacity: 0, y: 2 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}
              className="text-[11px]"
              style={{ color: feedback.type === "ok" ? "#34d399" : "#f87171" }}
            >
              {feedback.msg}
            </motion.p>
          )}
        </AnimatePresence>
      </Section>

      {/* Run analysis */}
      <Section icon={RefreshCw} title="Run Analysis">
        <Row label="Date range" description="Only analyse entries from this period">
          <Select value={daysBack} onValueChange={setDaysBack} disabled={analysis.phase === "running"}>
            <SelectTrigger className="w-[140px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DAY_RANGE_OPTIONS.map((o) => (
                <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Row>

        {/* Inline progress — switches between running / complete / error */}
        <AnimatePresence mode="wait">
          {analysis.phase === "running" && (
            <motion.div
              key="running"
              initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -4 }}
              className="space-y-2 pt-1"
            >
              <div className="flex items-center gap-2">
                <Loader2 size={13} className="text-[var(--accent)] flex-shrink-0" style={{ animation: "spin 1s linear infinite" }} />
                <span className="text-[11px] text-[var(--text-secondary)] leading-snug flex-1">{analysis.message}</span>
                <span className="text-[11px] font-medium text-[var(--accent)] flex-shrink-0">{analysis.percent}%</span>
              </div>
              <div style={{ height: 4, background: "var(--glass-control-bg)", borderRadius: 999, overflow: "hidden" }}>
                <motion.div
                  style={{ height: "100%", background: "var(--accent)", borderRadius: 999 }}
                  animate={{ width: `${analysis.percent}%` }}
                  transition={{ duration: 0.4, ease: "easeOut" }}
                />
              </div>
            </motion.div>
          )}

          {analysis.phase === "complete" && (
            <motion.div
              key="complete"
              initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}
              className="flex items-center gap-2 pt-1"
            >
              <Check size={13} className="text-emerald-400 flex-shrink-0" />
              <span className="text-[11px] text-emerald-400 flex-1">Analysis complete!</span>
              <button
                onClick={() => analysis.reportPath && handleOpenReport(analysis.reportPath)}
                className="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium"
                style={{ background: "var(--accent)", color: "#fff", border: "none", cursor: "pointer" }}
              >
                <ExternalLink size={10} /> Open Report
              </button>
            </motion.div>
          )}

          {analysis.phase === "error" && (
            <motion.div
              key="error"
              initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}
              className="pt-1"
            >
              <p className="text-[11px] text-red-400 leading-snug">{analysis.error}</p>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Action buttons */}
        <div className="flex gap-2 pt-1">
          <button
            onClick={handleRunAnalysis}
            disabled={!canRun}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[12px] font-semibold flex-1 justify-center transition-opacity"
            style={{
              background: canRun ? "var(--accent)" : "var(--glass-control-bg)",
              color: canRun ? "#fff" : "var(--text-tertiary)",
              border: "none",
              cursor: canRun ? "pointer" : "not-allowed",
              opacity: analysis.phase === "running" ? 0.5 : 1,
            }}
            title={count === 0 ? "Enable history and use the app first" : undefined}
          >
            {analysis.phase === "running"
              ? <><Loader2 size={13} style={{ animation: "spin 1s linear infinite" }} /> Running…</>
              : <><BarChart3 size={13} /> {analysis.phase === "complete" ? "Run Again" : "Run Analysis"}</>
            }
          </button>
          <button
            onClick={handleOpenFolder}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[12px] font-semibold"
            style={{ background: "var(--glass-control-bg)", border: "1px solid var(--chip-border)", color: "var(--text-secondary)", cursor: "pointer" }}
            title="Open reports folder"
          >
            <FolderOpen size={13} />
          </button>
        </div>
      </Section>

      {/* Past reports */}
      {reports.length > 0 && (
        <Section icon={ExternalLink} title="Past Reports">
          <div className="space-y-1">
            {reports.map((r) => (
              <div
                key={r.path}
                className="flex items-center justify-between gap-2 px-2 py-1.5 rounded-lg"
                style={{ background: "var(--glass-control-bg)" }}
              >
                <span className="text-[11px] text-[var(--text-secondary)] truncate flex-1">
                  {formatReportName(r.filename)}
                </span>
                <button
                  onClick={() => handleOpenReport(r.path)}
                  style={{ background: "none", border: "none", cursor: "pointer", color: "var(--accent)", padding: 2, flexShrink: 0 }}
                >
                  <ExternalLink size={12} />
                </button>
              </div>
            ))}
          </div>
        </Section>
      )}

      <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}

// ── General tab ───────────────────────────────────────────────────────────────

function GeneralTab() {
  const s = useContext(SettingContext);
  const { theme, setTheme } = useTheme();
  const [localApiKey, setLocalApiKey] = useState("");
  const [localPrompts, setLocalPrompts] = useState({ translate: "", correct: "", refine: "" });
  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "success" | "error">("idle");

  const isOllama = s.provider?.name === "ollama";

  useEffect(() => { if (s.apiKey) setLocalApiKey(s.apiKey); }, [s.apiKey]);
  useEffect(() => {
    setLocalPrompts({
      translate: s.prompts?.translate ?? "",
      correct: s.prompts?.correct ?? "",
      refine: s.prompts?.refine ?? "",
    });
  }, [s.prompts]);

  const handleSave = async () => {
    setIsSaving(true);
    setSaveStatus("idle");
    try {
      s.setPrompts(localPrompts);
      const ok = await s.saveSettings(localApiKey || undefined, localPrompts);
      setSaveStatus(ok ? "success" : "error");
    } catch { setSaveStatus("error"); }
    finally {
      setIsSaving(false);
      setTimeout(() => setSaveStatus("idle"), 3000);
    }
  };

  return (
    <div className="space-y-4">
      <Section icon={Bot} title="AI Provider">
        <Row label="Provider">
          <Select
            value={s.provider?.name}
            onValueChange={(name) => {
              const p = providerMap[name];
              s.setProvider(p);
              if (p.models) s.setModel(p.models[0]);
            }}
          >
            <SelectTrigger className="w-[140px]"><SelectValue placeholder="Select provider" /></SelectTrigger>
            <SelectContent>
              {Object.values(providerMap).map((p) => (
                <SelectItem key={p.name} value={p.name}>{p.label}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Row>
        <Input
          label="Model"
          value={s.model ?? ""}
          onChange={(e) => s.setModel(e.target.value)}
          placeholder="Enter model name"
          description={`Suggestions: ${s.provider?.models?.join(", ")}`}
        />
      </Section>

      <Section icon={Key} title="Authentication">
        <Input
          label="Base URL"
          value={s.modelUrl ?? ""}
          onChange={(e) => s.setModelUrl(e.target.value)}
          placeholder={isOllama ? "http://localhost:11434" : "Leave empty to use default"}
        />
        {!isOllama && (
          <Input
            label="API Key"
            type="password"
            value={localApiKey}
            onChange={(e) => setLocalApiKey(e.target.value)}
            placeholder="sk-..."
          />
        )}
      </Section>

      <Section icon={Zap} title="Inference">
        <Row label="Enable Thinking" description="Extended reasoning for models that support it">
          <Switch checked={s.thinking ?? true} onCheckedChange={(v) => s.setThinking(v)} />
        </Row>
      </Section>

      <Section icon={Monitor} title="Shortcut Behavior">
        <Row label="Window Type" description="Which window opens on shortcut trigger">
          <Select value={s.shortcutWindowType} onValueChange={(v) => s.setShortcutWindowType(v as ShortcutWindowType)}>
            <SelectTrigger className="w-[140px]"><SelectValue placeholder="Select" /></SelectTrigger>
            <SelectContent>
              <SelectItem value="popup">Small Popup</SelectItem>
              <SelectItem value="main">Main Window</SelectItem>
            </SelectContent>
          </Select>
        </Row>
      </Section>

      <Section icon={Languages} title="Language">
        <Row label="Preferred Language" description="Your native language — also the default translation target">
          <Select value={s.preferredLang ?? "Tiếng Việt"} onValueChange={(v) => s.setPreferredLang(v)}>
            <SelectTrigger className="w-[140px]"><SelectValue placeholder="Select language" /></SelectTrigger>
            <SelectContent>
              {LANGUAGES.map((l) => (<SelectItem key={l.value} value={l.label}>{l.label}</SelectItem>))}
            </SelectContent>
          </Select>
        </Row>
      </Section>

      <Section icon={MessageSquare} title="Prompts">
        <p className="text-[11px] text-[var(--text-tertiary)] -mt-1">
          Variables: <code className="text-[var(--text-secondary)]">{"{original_lang}"}</code>,{" "}
          <code className="text-[var(--text-secondary)]">{"{target_lang}"}</code>. Leave blank for defaults.
        </p>
        {(["translate", "correct", "refine"] as const).map((mode) => (
          <div key={mode} className="space-y-1.5">
            <p className="text-sm capitalize text-[var(--text-primary)]">{mode}</p>
            <textarea
              value={localPrompts[mode]}
              onChange={(e) => setLocalPrompts((p) => ({ ...p, [mode]: e.target.value }))}
              placeholder={DEFAULT_PROMPTS[mode]}
              rows={3}
              className={cn(
                "w-full resize-none text-xs rounded-lg px-3 py-2 leading-relaxed",
                "bg-[var(--glass-input-bg)] border border-[var(--glass-border)]",
                "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)]",
                "focus:outline-none focus:border-[var(--glass-border-bright)] scrollbar-hide transition-colors",
              )}
            />
          </div>
        ))}
      </Section>

      <Section icon={Palette} title="Appearance">
        <Row label="Theme">
          <Select value={theme} onValueChange={(v) => setTheme(v as ThemeType)}>
            <SelectTrigger className="w-[140px]"><SelectValue placeholder="Select theme" /></SelectTrigger>
            <SelectContent>
              <SelectItem value="system">System</SelectItem>
              <SelectItem value="dark">Dark</SelectItem>
              <SelectItem value="light">Light</SelectItem>
            </SelectContent>
          </Select>
        </Row>
        <Row label="Text Size" description="Scales all text across windows">
          <Select value={s.textSize ?? "medium"} onValueChange={(v) => s.setTextSize(v as TextSizeType)}>
            <SelectTrigger className="w-[140px]"><SelectValue placeholder="Select size" /></SelectTrigger>
            <SelectContent>
              <SelectItem value="small">Small</SelectItem>
              <SelectItem value="medium">Medium</SelectItem>
              <SelectItem value="large">Large</SelectItem>
            </SelectContent>
          </Select>
        </Row>
      </Section>

      <Button onClick={handleSave} disabled={isSaving} className="w-full gap-1.5">
        <AnimatePresence mode="wait">
          {saveStatus === "success" ? (
            <motion.div key="ok" initial={{ scale: 0 }} animate={{ scale: 1 }} className="flex items-center gap-1.5">
              <Check size={13} /> Saved
            </motion.div>
          ) : (
            <motion.div key="save" className="flex items-center gap-1.5">
              {isSaving
                ? <div className="h-3 w-3 rounded-full border-2 border-white/40 border-t-white animate-spin" />
                : <Save size={13} />}
              {isSaving ? "Saving…" : "Save Settings"}
            </motion.div>
          )}
        </AnimatePresence>
      </Button>

      <p className="text-center text-[10px] text-[var(--text-tertiary)] pb-2">
        © 2026 Refiner App
      </p>
    </div>
  );
}

// ── Root ──────────────────────────────────────────────────────────────────────

type Tab = "general" | "analysis";
const TABS: { id: Tab; label: string }[] = [
  { id: "general", label: "General" },
  { id: "analysis", label: "Analysis" },
];

export default function SettingsView() {
  const initialTab = (new URLSearchParams(window.location.search).get("tab") as Tab) || "general";
  const [activeTab, setActiveTab] = useState<Tab>(initialTab);

  // For when the window is already open and tray fires "Analysis"
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("show-analysis-tab", () => setActiveTab("analysis")).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  return (
    <div style={{
      width: "100%", height: "100vh", background: "transparent",
      display: "flex", borderRadius: "0 0 var(--radius-app) var(--radius-app)", overflow: "hidden",
    }}>
      {/* Glass surface */}
      <div style={{
        flex: 1, display: "flex", flexDirection: "column", overflow: "hidden",
        background: "var(--glass-app-bg)",
        backdropFilter: "blur(40px) saturate(150%)",
        WebkitBackdropFilter: "blur(40px) saturate(150%)",
        boxShadow: "var(--glass-shadow)",
        position: "relative",
        borderRadius: "0 0 var(--radius-app) var(--radius-app)",
      }}>
        {/* Noise texture */}
        <div style={{ position: "absolute", inset: 0, backgroundImage: "var(--noise-url)", opacity: 0.03, pointerEvents: "none", zIndex: 0 }} />
        {/* Specular highlight */}
        <div style={{ position: "absolute", top: 0, left: "10%", right: "10%", height: 1, background: "linear-gradient(90deg, transparent, rgba(255,255,255,0.12), transparent)", pointerEvents: "none", zIndex: 1 }} />

        {/* ── Tab bar ── */}
        <div style={{
          padding: "10px 14px 10px", flexShrink: 0,
          borderBottom: "1px solid var(--glass-border)",
          position: "relative", zIndex: 2,
        }}>
          <div
            role="tablist"
            style={{
              display: "inline-flex", alignItems: "center", gap: 2,
              borderRadius: 12, padding: 3,
              background: "var(--glass-control-bg)",
              border: "1px solid var(--chip-border)",
              boxShadow: "var(--chip-highlight)",
            }}
          >
            {TABS.map(({ id, label }) => {
              const active = id === activeTab;
              return (
                <button
                  key={id}
                  role="tab"
                  aria-selected={active}
                  onClick={() => setActiveTab(id)}
                  style={{
                    position: "relative", padding: "4px 14px", borderRadius: 8,
                    border: "none", cursor: "pointer", background: "transparent",
                    fontSize: 12, fontWeight: 600,
                    color: active ? "var(--text-primary)" : "var(--text-tertiary)",
                    transition: "color 0.2s",
                  }}
                >
                  {active && (
                    <motion.div
                      layoutId="settings-tab-pill"
                      style={{
                        position: "absolute", inset: 0, borderRadius: 8,
                        background: "var(--glass-control-active)",
                        border: "1px solid var(--chip-border)",
                        boxShadow: "inset 0 1px 0 rgba(255,255,255,0.14), 0 1px 4px rgba(0,0,0,0.25)",
                      }}
                      transition={{ type: "spring", stiffness: 500, damping: 35 }}
                    />
                  )}
                  <span style={{ position: "relative", zIndex: 1 }}>{label}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Scrollable content */}
        <div
          style={{ flex: 1, overflowY: "auto", padding: "16px 16px", position: "relative", zIndex: 2 }}
          className="scrollbar-hide"
        >
          <AnimatePresence mode="wait">
            {activeTab === "general" ? (
              <motion.div
                key="general"
                initial={{ opacity: 0, x: -8 }} animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 8 }} transition={{ duration: 0.15 }}
              >
                <GeneralTab />
              </motion.div>
            ) : (
              <motion.div
                key="analysis"
                initial={{ opacity: 0, x: 8 }} animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -8 }} transition={{ duration: 0.15 }}
              >
                <AnalysisTab />
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
