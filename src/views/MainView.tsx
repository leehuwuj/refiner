import { useCallback, useContext, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { motion, AnimatePresence } from "framer-motion";
import {
  ArrowRight,
  Square,
  Settings,
  Copy,
  Check,
  ArrowLeftRight,
  Sparkles,
} from "lucide-react";

import { TranslateContext } from "@/providers/translate";
import { SettingContext } from "@/providers/settings";
import { Tooltip } from "@/components/ui/tooltip";
import { SkeletonText } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Mode } from "@/types/translate";

// ── Tauri invoke ──────────────────────────────────────────────────────────────

async function runMode(
  text: string,
  mode: Mode,
  provider?: string,
  model?: string,
  sourceLang?: string,
  targetLang?: string,
  customPrompt?: string,
): Promise<string> {
  const fn = mode.toLowerCase();
  return invoke<string>(fn, {
    provider: provider || null,
    model: model || null,
    text,
    sourceLang: sourceLang ?? "English",
    targetLang: targetLang ?? "Tiếng Việt",
    prompt: customPrompt || null,
  });
}

// ── Mode Tabs ─────────────────────────────────────────────────────────────────

const MODES: Mode[] = ["Translate", "Correct", "Refine"];

function ModeTabs({
  current,
  onChange,
}: {
  current: Mode;
  onChange: (mode: Mode) => void;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 1,
        borderRadius: 10,
        padding: 3,
        background: "var(--glass-control-bg)",
      }}
    >
      {MODES.map((mode) => {
        const active = mode === current;
        return (
          <button
            key={mode}
            onClick={() => onChange(mode)}
            style={{
              position: "relative",
              padding: "4px 14px",
              fontSize: 11,
              fontWeight: 500,
              borderRadius: 7,
              border: "none",
              cursor: "pointer",
              background: "transparent",
              color: active ? "var(--text-primary)" : "var(--text-tertiary)",
              transition: "color 0.2s",
              userSelect: "none",
            }}
          >
            {active && (
              <motion.div
                layoutId="mode-pill"
                style={{
                  position: "absolute",
                  inset: 0,
                  borderRadius: 7,
                  background: "var(--glass-control-hover)",
                  boxShadow:
                    "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 3px rgba(0,0,0,0.2)",
                }}
                transition={{ type: "spring", stiffness: 500, damping: 35 }}
              />
            )}
            <span style={{ position: "relative", zIndex: 1 }}>{mode}</span>
          </button>
        );
      })}
    </div>
  );
}

// ── Language Bar ──────────────────────────────────────────────────────────────

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

function LanguageBar({
  sourceLang,
  targetLang,
  onSwap,
  onTargetChange,
}: {
  sourceLang: string;
  targetLang: string;
  onSwap: () => void;
  onTargetChange: (val: string) => void;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <span
        style={{
          fontSize: 10,
          color: "var(--text-secondary)",
          fontWeight: 500,
          padding: "2px 8px",
          borderRadius: 6,
          background: "var(--glass-control-bg)",
        }}
      >
        {sourceLang}
      </span>
      <button
        onClick={onSwap}
        className="hover:text-[var(--text-primary)]"
        style={{
          padding: 4,
          borderRadius: 6,
          border: "none",
          background: "transparent",
          color: "var(--text-tertiary)",
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          transition: "color 0.15s",
        }}
      >
        <ArrowLeftRight size={11} />
      </button>
      <Select value={targetLang} onValueChange={onTargetChange}>
        <SelectTrigger compact className="w-auto min-w-[80px] text-[10px]">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {LANGUAGES.map((l) => (
            <SelectItem key={l.value} value={l.label} className="text-xs">
              {l.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

// ── Copy Button ───────────────────────────────────────────────────────────────

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Tooltip content={copied ? "Copied!" : "Copy"} open={copied || undefined}>
      <button
        onClick={handleCopy}
        className="hover:bg-[var(--glass-control-hover)] hover:text-[var(--text-primary)]"
        style={{
          padding: 6,
          borderRadius: 8,
          border: "none",
          background: "var(--glass-control-bg)",
          color: "var(--text-tertiary)",
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          transition: "all 0.15s",
        }}
      >
        <AnimatePresence mode="wait">
          {copied ? (
            <motion.div
              key="check"
              initial={{ scale: 0.5, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.5, opacity: 0 }}
              transition={{ duration: 0.15 }}
            >
              <Check size={13} />
            </motion.div>
          ) : (
            <motion.div
              key="copy"
              initial={{ scale: 1 }}
              animate={{ scale: 1 }}
            >
              <Copy size={13} />
            </motion.div>
          )}
        </AnimatePresence>
      </button>
    </Tooltip>
  );
}

// ── Main View ─────────────────────────────────────────────────────────────────

export default function MainView() {
  const ctx = useContext(TranslateContext);
  const settings = useContext(SettingContext);
  const [triggerByShortcut, setTriggerByShortcut] = useState(false);
  const [styleHint, setStyleHint] = useState("");
  const [loading, setLoading] = useState(false);
  const isMounted = useRef(false);

  // Sync preferred language from settings into target language
  useEffect(() => {
    if (settings.preferredLang) {
      const lang = LANGUAGES.find((l) => l.label === settings.preferredLang);
      if (lang) {
        ctx.changeLangConfig({
          ...ctx.languageConfig,
          targetLang: { code: lang.value, label: lang.label },
        });
      }
    }
  }, [settings.preferredLang]);

  const trigger = useCallback(() => {
    if (loading) return;
    setLoading(true);
    ctx.setTranslating(true);
    ctx.changeResult({});
    const modeKey =
      ctx.currentMode.toLowerCase() as keyof typeof settings.prompts;
    runMode(
      ctx.inputText ?? "",
      ctx.currentMode,
      settings.provider?.name,
      settings.model,
      ctx.languageConfig.sourceLang.label,
      ctx.languageConfig.targetLang.label,
      settings.prompts?.[modeKey],
    )
      .then((answer) => {
        ctx.changeResult({ [ctx.currentMode.toLowerCase()]: answer });
      })
      .catch(console.error)
      .finally(() => {
        setLoading(false);
        ctx.setTranslating(false);
      });
  }, [loading, ctx, settings]);

  useEffect(() => {
    if (!isMounted.current) {
      isMounted.current = true;
      listen("shortcut-main-translate", (event) => {
        let raw = event.payload as string;
        if (raw.includes("text:")) raw = raw.split("text:")[1].trim();
        raw = raw.replace(/^"|"$/g, "").trim();
        window.dispatchEvent(
          new CustomEvent("shortcut-main-translate", { detail: raw }),
        );
      });
    }
  }, []);

  useEffect(() => {
    const handler = (e: Event) => {
      ctx.changeInputText((e as CustomEvent).detail);
      setTriggerByShortcut(true);
    };
    window.addEventListener("shortcut-main-translate", handler);
    return () => window.removeEventListener("shortcut-main-translate", handler);
  }, [ctx]);

  useEffect(() => {
    if (triggerByShortcut) {
      setTriggerByShortcut(false);
      trigger();
    }
  }, [triggerByShortcut, trigger]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        window.__TAURI__.window.getCurrentWindow().hide();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter" && !loading) {
        trigger();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [loading, trigger]);

  const handleModeChange = useCallback(
    (mode: Mode) => {
      ctx.setCurrentMode(mode);
      if (ctx.inputText?.trim()) {
        setLoading(true);
        ctx.setTranslating(true);
        ctx.changeResult({});
        const modeKey = mode.toLowerCase() as keyof typeof settings.prompts;
        runMode(
          ctx.inputText,
          mode,
          settings.provider?.name,
          settings.model,
          ctx.languageConfig.sourceLang.label,
          ctx.languageConfig.targetLang.label,
          settings.prompts?.[modeKey],
        )
          .then((answer) => ctx.changeResult({ [mode.toLowerCase()]: answer }))
          .catch(console.error)
          .finally(() => {
            setLoading(false);
            ctx.setTranslating(false);
          });
      }
    },
    [ctx, settings],
  );

  const currentResult =
    ctx.result?.[ctx.currentMode.toLowerCase() as keyof typeof ctx.result] ??
    "";

  const handleSwapLanguages = () => {
    ctx.changeLangConfig({
      sourceLang: ctx.languageConfig.targetLang,
      targetLang: ctx.languageConfig.sourceLang,
    });
  };

  const handleTargetLangChange = (label: string) => {
    const lang = LANGUAGES.find((l) => l.label === label);
    if (lang) {
      ctx.changeLangConfig({
        ...ctx.languageConfig,
        targetLang: { code: lang.value, label: lang.label },
      });
    }
  };

  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        background: "transparent",
        display: "flex",
        borderRadius: "var(--radius-app)",
        overflow: "hidden",
      }}
    >
      {/* ── Main surface ── */}
      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          background: "var(--glass-app-bg)",
          backdropFilter: "blur(40px) saturate(150%)",
          WebkitBackdropFilter: "blur(40px) saturate(150%)",
          boxShadow: "var(--glass-shadow)",
          borderRadius: "var(--radius-app)",
        }}
      >
        {/* Noise texture overlay */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            backgroundImage: "var(--noise-url)",
            opacity: 0.03,
            pointerEvents: "none",
            zIndex: 0,
          }}
        />
        {/* Specular highlight — top edge */}
        <div
          style={{
            position: "absolute",
            top: 0,
            left: "8%",
            right: "8%",
            height: 1,
            background:
              "linear-gradient(90deg, transparent, rgba(255,255,255,0.12), transparent)",
            pointerEvents: "none",
            zIndex: 1,
          }}
        />

        {/* ── Header ── */}
        <div
          data-tauri-drag-region
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "8px 12px",
            flexShrink: 0,
            position: "relative",
            zIndex: 2,
          }}
        >
          <ModeTabs current={ctx.currentMode} onChange={handleModeChange} />

          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            {settings.provider && (
              <Select
                value={settings.model}
                onValueChange={(val) => settings.setModel(val)}
              >
                <SelectTrigger
                  compact
                  className="w-auto max-w-[140px] text-[10px]"
                >
                  <SelectValue placeholder="Model" />
                </SelectTrigger>
                <SelectContent>
                  {[
                    ...(settings.provider.models ?? []),
                    ...(settings.model &&
                    !settings.provider.models?.includes(settings.model)
                      ? [settings.model]
                      : []),
                  ].map((m) => (
                    <SelectItem key={m} value={m} className="text-xs">
                      {m}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}

            <Tooltip content="Settings">
              <button
                onClick={() => invoke("open_settings_window")}
                className="hover:bg-[var(--glass-control-hover)] hover:text-[var(--text-primary)]"
                style={{
                  padding: 6,
                  borderRadius: 8,
                  border: "none",
                  background: "transparent",
                  color: "var(--text-tertiary)",
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  transition: "all 0.15s",
                }}
              >
                <Settings size={14} />
              </button>
            </Tooltip>
          </div>
        </div>

        {/* ── Body ── */}
        <div
          style={{
            flex: 1,
            display: "flex",
            minHeight: 0,
            gap: 0,
            position: "relative",
            zIndex: 2,
          }}
        >
          {/* Input panel */}
          <div
            style={{
              flex: 1,
              display: "flex",
              flexDirection: "column",
              overflow: "hidden",
              margin: "0 0 0 10px",
              marginBottom: 8,
              borderRadius: 12,
              background:
                "linear-gradient(180deg, var(--glass-panel-bg) 0%, transparent 100%)",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.05)",
            }}
          >
            <textarea
              value={ctx.inputText}
              onChange={(e) => ctx.changeInputText(e.target.value)}
              placeholder={
                "Paste or type text here...\n\nCtrl+Enter to process."
              }
              style={{
                flex: 1,
                background: "transparent",
                border: "none",
                outline: "none",
                resize: "none",
                padding: "12px 14px",
                fontSize: 13,
                lineHeight: 1.65,
                color: "var(--text-primary)",
                fontFamily: "inherit",
              }}
              className="scrollbar-hide"
            />
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                padding: "4px 12px 8px",
              }}
            >
              <span style={{ fontSize: 10, color: "var(--text-tertiary)" }}>
                {ctx.inputText ? `${ctx.inputText.length} chars` : ""}
              </span>
              {ctx.inputText && (
                <button
                  onClick={() => ctx.changeInputText("")}
                  className="hover:text-[var(--text-secondary)]"
                  style={{
                    fontSize: 10,
                    color: "var(--text-tertiary)",
                    background: "none",
                    border: "none",
                    cursor: "pointer",
                    transition: "color 0.15s",
                  }}
                >
                  Clear
                </button>
              )}
            </div>
          </div>

          {/* Action column */}
          <div
            style={{
              width: 44,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              flexShrink: 0,
            }}
          >
            <AnimatePresence mode="wait">
              {loading ? (
                <motion.button
                  key="stop"
                  initial={{ scale: 0.7, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.7, opacity: 0 }}
                  onClick={() => {
                    setLoading(false);
                    ctx.setTranslating(false);
                  }}
                  style={{
                    width: 30,
                    height: 30,
                    borderRadius: "50%",
                    border: "none",
                    background: "var(--glass-control-bg)",
                    color: "var(--text-secondary)",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06)",
                  }}
                >
                  <Square size={10} />
                </motion.button>
              ) : (
                <motion.button
                  key="run"
                  initial={{ scale: 0.7, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.7, opacity: 0 }}
                  onClick={trigger}
                  style={{
                    width: 30,
                    height: 30,
                    borderRadius: "50%",
                    border: "none",
                    background: "rgba(255,255,255,0.18)",
                    color: "var(--text-primary)",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.12)",
                  }}
                  whileHover={{ scale: 1.1 }}
                  whileTap={{ scale: 0.92 }}
                >
                  <ArrowRight size={13} strokeWidth={2.5} />
                </motion.button>
              )}
            </AnimatePresence>
          </div>

          {/* Output panel */}
          <div
            style={{
              flex: 1,
              display: "flex",
              flexDirection: "column",
              overflow: "hidden",
              margin: "0 10px 0 0",
              marginBottom: 8,
              borderRadius: 12,
              background:
                "linear-gradient(180deg, var(--glass-panel-bg) 0%, transparent 100%)",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.05)",
            }}
          >
            <div
              style={{
                flex: 1,
                padding: "12px 14px",
                overflowY: "auto",
                position: "relative",
              }}
              className="scrollbar-hide"
            >
              {loading ? (
                <motion.div
                  key="skeleton"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ duration: 0.15 }}
                >
                  <SkeletonText lines={4} />
                </motion.div>
              ) : currentResult ? (
                <motion.p
                  key="result"
                  initial={{ opacity: 0, y: 4 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.2 }}
                  style={{
                    fontSize: 13,
                    lineHeight: 1.65,
                    color: "var(--text-primary)",
                    margin: 0,
                    paddingRight: 28,
                  }}
                >
                  {currentResult}
                </motion.p>
              ) : (
                <motion.div
                  key="empty"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  style={{
                    height: "100%",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  <p
                    style={{
                      fontSize: 11,
                      color: "var(--text-tertiary)",
                      textAlign: "center",
                      lineHeight: 1.6,
                      maxWidth: 160,
                      margin: 0,
                    }}
                  >
                    Result will appear here
                  </p>
                </motion.div>
              )}

              {currentResult && (
                <div style={{ position: "absolute", top: 8, right: 8 }}>
                  <CopyButton text={currentResult} />
                </div>
              )}
            </div>

            <div
              style={{
                padding: "4px 12px 8px",
                minHeight: 28,
                display: "flex",
                alignItems: "center",
              }}
            >
              {ctx.currentMode === "Translate" && (
                <LanguageBar
                  sourceLang={ctx.languageConfig.sourceLang.label}
                  targetLang={ctx.languageConfig.targetLang.label}
                  onSwap={handleSwapLanguages}
                  onTargetChange={handleTargetLangChange}
                />
              )}
            </div>
          </div>
        </div>

        {/* ── Footer ── */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "4px 14px 8px",
            flexShrink: 0,
            position: "relative",
            zIndex: 2,
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 6,
              flex: 1,
              maxWidth: 240,
            }}
          >
            <Sparkles
              size={10}
              style={{ color: "var(--text-tertiary)", opacity: 0.6 }}
            />
            <input
              value={styleHint}
              onChange={(e) => setStyleHint(e.target.value)}
              placeholder="Writing style..."
              style={{
                flex: 1,
                fontSize: 11,
                background: "transparent",
                border: "none",
                outline: "none",
                color: "var(--text-secondary)",
                fontFamily: "inherit",
              }}
            />
          </div>

          <span
            style={{
              fontSize: 10,
              color: "var(--text-tertiary)",
              opacity: 0.6,
            }}
          >
            Ctrl+Enter
          </span>

          <span
            style={{
              fontSize: 10,
              color: "var(--text-tertiary)",
              fontWeight: 500,
              padding: "2px 8px",
              borderRadius: 6,
              background: "var(--glass-control-bg)",
            }}
          >
            {settings.provider?.label ?? "Refiner"}
          </span>
        </div>
      </div>
    </div>
  );
}
