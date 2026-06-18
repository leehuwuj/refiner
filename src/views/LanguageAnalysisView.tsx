import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { motion, AnimatePresence } from "framer-motion";
import { BarChart3, ExternalLink, Loader2, CheckCircle2, AlertCircle, RefreshCw } from "lucide-react";

// ── Types ──────────────────────────────────────────────────────────────────────

interface AnalysisStatus {
  running: boolean;
  message: string;
  percent: number;
  complete: boolean;
  report_path: string | null;
  error: string | null;
}

type Phase = "idle" | "running" | "complete" | "error";

// ── Progress bar ───────────────────────────────────────────────────────────────

function ProgressBar({ percent }: { percent: number }) {
  return (
    <div
      style={{
        width: "100%",
        height: 6,
        borderRadius: 4,
        background: "var(--glass-control-bg)",
        overflow: "hidden",
      }}
    >
      <motion.div
        style={{
          height: "100%",
          borderRadius: 4,
          background: "linear-gradient(90deg, var(--accent), #a78bfa)",
        }}
        initial={{ width: 0 }}
        animate={{ width: `${percent}%` }}
        transition={{ duration: 0.4, ease: "easeOut" }}
      />
    </div>
  );
}

// ── Main view ──────────────────────────────────────────────────────────────────

export default function LanguageAnalysisView() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [message, setMessage] = useState("Ready to analyse your language samples.");
  const [percent, setPercent] = useState(0);
  const [reportPath, setReportPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Sync from server-side state on mount (catches a run that started before
  // the window's JS was loaded).
  useEffect(() => {
    invoke<AnalysisStatus>("get_analysis_status").then((s) => {
      applyStatus(s);
    });

    const unlistenProgress = listen<{ message: string; percent: number }>(
      "analysis-progress",
      ({ payload }) => {
        setPhase("running");
        setMessage(payload.message);
        setPercent(payload.percent);
        setError(null);
      }
    );

    const unlistenComplete = listen<{ path: string }>(
      "analysis-complete",
      ({ payload }) => {
        setPhase("complete");
        setMessage("Analysis complete!");
        setPercent(100);
        setReportPath(payload.path);
        setError(null);
      }
    );

    const unlistenError = listen<{ error: string }>(
      "analysis-error",
      ({ payload }) => {
        setPhase("error");
        setError(payload.error);
      }
    );

    return () => {
      unlistenProgress.then((f) => f());
      unlistenComplete.then((f) => f());
      unlistenError.then((f) => f());
    };
  }, []);

  function applyStatus(s: AnalysisStatus) {
    if (s.complete) {
      setPhase("complete");
      setMessage("Analysis complete!");
      setPercent(100);
      setReportPath(s.report_path);
    } else if (s.error) {
      setPhase("error");
      setError(s.error);
    } else if (s.running) {
      setPhase("running");
      setMessage(s.message || "Running…");
      setPercent(s.percent);
    }
  }

  async function handleOpenReport() {
    try {
      await invoke("open_last_report");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRunAgain() {
    setPhase("running");
    setMessage("Starting analysis…");
    setPercent(0);
    setError(null);
    try {
      await invoke("run_language_analysis");
    } catch (e) {
      setPhase("error");
      setError(String(e));
    }
  }

  return (
    <div
      style={{
        width: "100%",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        background: "var(--glass-app-bg)",
        color: "var(--text-primary)",
        fontFamily: "var(--font-sans, system-ui)",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          padding: "20px 24px 16px",
          borderBottom: "1px solid var(--glass-border)",
          flexShrink: 0,
        }}
      >
        <div
          style={{
            width: 36,
            height: 36,
            borderRadius: 10,
            background: "var(--accent-subtle)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexShrink: 0,
          }}
        >
          <BarChart3 size={18} style={{ color: "var(--accent)" }} />
        </div>
        <div>
          <h1
            style={{
              fontSize: 15,
              fontWeight: 700,
              margin: 0,
              color: "var(--text-primary)",
            }}
          >
            Language Analysis
          </h1>
          <p
            style={{
              fontSize: 11,
              color: "var(--text-tertiary)",
              margin: 0,
              marginTop: 1,
            }}
          >
            AI-powered L2 proficiency assessment
          </p>
        </div>
      </div>

      {/* Body */}
      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "32px 32px",
          gap: 24,
        }}
      >
        <AnimatePresence mode="wait">
          {/* ── Running ── */}
          {phase === "running" && (
            <motion.div
              key="running"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              style={{ width: "100%", maxWidth: 420, display: "flex", flexDirection: "column", gap: 16 }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                <Loader2
                  size={18}
                  style={{ color: "var(--accent)", flexShrink: 0, animation: "spin 1s linear infinite" }}
                />
                <span style={{ fontSize: 13, fontWeight: 600, color: "var(--text-primary)" }}>
                  Analysing…
                </span>
              </div>

              <ProgressBar percent={percent} />

              <p
                style={{
                  fontSize: 12,
                  color: "var(--text-secondary)",
                  margin: 0,
                  lineHeight: 1.5,
                }}
              >
                {message}
              </p>

              <p
                style={{
                  fontSize: 11,
                  color: "var(--text-tertiary)",
                  margin: 0,
                }}
              >
                This window will stay open. The report opens automatically when done.
              </p>
            </motion.div>
          )}

          {/* ── Complete ── */}
          {phase === "complete" && (
            <motion.div
              key="complete"
              initial={{ opacity: 0, scale: 0.97 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0 }}
              style={{ width: "100%", maxWidth: 420, display: "flex", flexDirection: "column", gap: 20, alignItems: "center" }}
            >
              <div
                style={{
                  width: 56,
                  height: 56,
                  borderRadius: "50%",
                  background: "rgba(52,211,153,0.12)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <CheckCircle2 size={28} style={{ color: "#34d399" }} />
              </div>

              <div style={{ textAlign: "center" }}>
                <p style={{ fontSize: 15, fontWeight: 700, margin: 0, color: "var(--text-primary)" }}>
                  Report ready
                </p>
                <p style={{ fontSize: 12, color: "var(--text-secondary)", margin: "6px 0 0" }}>
                  Your language assessment has been saved and opened in your browser.
                </p>
              </div>

              <div style={{ display: "flex", gap: 8 }}>
                <button
                  onClick={handleOpenReport}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    padding: "8px 16px",
                    borderRadius: 8,
                    background: "var(--accent)",
                    color: "#fff",
                    border: "none",
                    fontSize: 12,
                    fontWeight: 600,
                    cursor: "pointer",
                  }}
                >
                  <ExternalLink size={13} />
                  Open Report
                </button>
                <button
                  onClick={handleRunAgain}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    padding: "8px 16px",
                    borderRadius: 8,
                    background: "var(--glass-control-bg)",
                    color: "var(--text-secondary)",
                    border: "1px solid var(--chip-border)",
                    fontSize: 12,
                    fontWeight: 600,
                    cursor: "pointer",
                  }}
                >
                  <RefreshCw size={13} />
                  Run Again
                </button>
              </div>
            </motion.div>
          )}

          {/* ── Error ── */}
          {phase === "error" && (
            <motion.div
              key="error"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              style={{ width: "100%", maxWidth: 420, display: "flex", flexDirection: "column", gap: 16, alignItems: "center" }}
            >
              <div
                style={{
                  width: 56,
                  height: 56,
                  borderRadius: "50%",
                  background: "rgba(248,113,113,0.12)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <AlertCircle size={28} style={{ color: "#f87171" }} />
              </div>

              <div style={{ textAlign: "center" }}>
                <p style={{ fontSize: 14, fontWeight: 700, margin: 0, color: "var(--text-primary)" }}>
                  Analysis failed
                </p>
                <p
                  style={{
                    fontSize: 12,
                    color: "var(--text-secondary)",
                    margin: "6px 0 0",
                    lineHeight: 1.5,
                  }}
                >
                  {error}
                </p>
              </div>

              <button
                onClick={handleRunAgain}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  padding: "8px 16px",
                  borderRadius: 8,
                  background: "var(--glass-control-bg)",
                  color: "var(--text-secondary)",
                  border: "1px solid var(--chip-border)",
                  fontSize: 12,
                  fontWeight: 600,
                  cursor: "pointer",
                }}
              >
                <RefreshCw size={13} />
                Try Again
              </button>
            </motion.div>
          )}

          {/* ── Idle ── */}
          {phase === "idle" && (
            <motion.div
              key="idle"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 12 }}
            >
              <BarChart3 size={40} style={{ color: "var(--text-tertiary)" }} />
              <p style={{ fontSize: 13, color: "var(--text-secondary)", margin: 0, textAlign: "center" }}>
                {message}
              </p>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <style>{`
        @keyframes spin { to { transform: rotate(360deg); } }
      `}</style>
    </div>
  );
}
