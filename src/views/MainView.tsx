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
        gap: 2,
        borderRadius: 12,
        padding: 4,
        background: "var(--glass-control-bg)",
        border: "1px solid var(--chip-border)",
        boxShadow: "var(--chip-highlight)",
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
              padding: "6px 16px",
              fontSize: 12,
              fontWeight: 600,
              borderRadius: 8,
              border: "none",
              cursor: "pointer",
              background: "transparent",
              color: active ? "var(--text-primary)" : "var(--text-secondary)",
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
                  borderRadius: 8,
                  background: "var(--glass-control-active)",
                  border: "1px solid var(--chip-border)",
                  boxShadow:
                    "inset 0 1px 0 rgba(255,255,255,0.14), 0 1px 4px rgba(0,0,0,0.25)",
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
          display: "inline-flex",
          alignItems: "center",
          height: 32,
          fontSize: 12,
          color: "var(--text-secondary)",
          fontWeight: 600,
          padding: "0 12px",
          borderRadius: 8,
          whiteSpace: "nowrap",
          background: "var(--glass-control-bg)",
          border: "1px solid var(--chip-border)",
          boxShadow: "var(--chip-highlight)",
        }}
      >
        {sourceLang}
      </span>
      <button
        onClick={onSwap}
        title="Swap languages"
        className="lg-chip"
        style={{
          width: 32,
          height: 32,
          borderRadius: "50%",
          flexShrink: 0,
        }}
      >
        <ArrowLeftRight size={13} />
      </button>
      <Select value={targetLang} onValueChange={onTargetChange}>
        <SelectTrigger compact className="w-auto min-w-[96px]">
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
        className="lg-chip"
        style={{
          width: 30,
          height: 30,
          borderRadius: 9,
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
              <Check size={14} />
            </motion.div>
          ) : (
            <motion.div
              key="copy"
              initial={{ scale: 1 }}
              animate={{ scale: 1 }}
            >
              <Copy size={14} />
            </motion.div>
          )}
        </AnimatePresence>
      </button>
    </Tooltip>
  );
}

function truncateMiddle(str: string, max = 22): string {
  if (str.length <= max) return str;
  const head = Math.ceil((max - 1) / 2);
  const tail = Math.floor((max - 1) / 2);
  return str.slice(0, head) + "…" + str.slice(-tail);
}

// ── Main View ─────────────────────────────────────────────────────────────────

export default function MainView() {
  const ctx = useContext(TranslateContext);
  const settings = useContext(SettingContext);
  const [triggerByShortcut, setTriggerByShortcut] = useState(false);
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
            {settings.model && (
              <span
                style={{
                  fontSize: 10,
                  color: "var(--text-secondary)",
                  fontWeight: 600,
                  padding: "5px 10px",
                  borderRadius: 8,
                  background: "var(--glass-control-bg)",
                  border: "1px solid var(--chip-border)",
                  boxShadow: "var(--chip-highlight)",
                  whiteSpace: "nowrap",
                  display: "inline-block",
                }}
                title={settings.model}
              >
                {truncateMiddle(settings.model)}
              </span>
            )}

            <Tooltip content="Settings">
              <button
                onClick={() => invoke("open_settings_window")}
                className="lg-chip"
                style={{
                  width: 32,
                  height: 32,
                  borderRadius: "50%",
                }}
              >
                <Settings size={15} />
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
              borderRadius: 14,
              background:
                "linear-gradient(180deg, var(--glass-panel-bg) 0%, transparent 100%)",
              border: "1px solid var(--glass-border)",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06)",
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
              width: 56,
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
                  className="lg-chip"
                  initial={{ scale: 0.7, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.7, opacity: 0 }}
                  onClick={() => {
                    setLoading(false);
                    ctx.setTranslating(false);
                  }}
                  style={{
                    width: 42,
                    height: 42,
                    borderRadius: "50%",
                    color: "var(--text-primary)",
                  }}
                  whileHover={{ scale: 1.06 }}
                  whileTap={{ scale: 0.92 }}
                >
                  <Square size={13} fill="currentColor" />
                </motion.button>
              ) : (
                <motion.button
                  key="run"
                  className="lg-chip-primary"
                  initial={{ scale: 0.7, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.7, opacity: 0 }}
                  onClick={trigger}
                  style={{
                    width: 42,
                    height: 42,
                    borderRadius: "50%",
                  }}
                  whileHover={{ scale: 1.08 }}
                  whileTap={{ scale: 0.92 }}
                >
                  <ArrowRight size={18} strokeWidth={2.5} />
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
              borderRadius: 14,
              background:
                "linear-gradient(180deg, var(--glass-panel-bg) 0%, transparent 100%)",
              border: "1px solid var(--glass-border)",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06)",
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
          </div>
        </div>

        {/* ── Footer ── */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr auto 1fr",
            alignItems: "center",
            padding: "6px 14px 10px",
            flexShrink: 0,
            position: "relative",
            zIndex: 2,
          }}
        >
          {/* Left: keyboard hint */}
          <div style={{ justifySelf: "start" }}>
            <span
              style={{
                fontSize: 11,
                fontWeight: 600,
                color: "var(--text-secondary)",
                padding: "5px 10px",
                borderRadius: 8,
                background: "var(--glass-control-bg)",
                border: "1px solid var(--chip-border)",
                boxShadow: "var(--chip-highlight)",
                whiteSpace: "nowrap",
              }}
            >
              ⌃⏎
            </span>
          </div>

          {/* Center: language options (translate mode only) */}
          <div style={{ justifySelf: "center" }}>
            {ctx.currentMode === "Translate" && (
              <LanguageBar
                sourceLang={ctx.languageConfig.sourceLang.label}
                targetLang={ctx.languageConfig.targetLang.label}
                onSwap={handleSwapLanguages}
                onTargetChange={handleTargetLangChange}
              />
            )}
          </div>

          {/* Right: provider badge */}
          <span
            style={{
              justifySelf: "end",
              fontSize: 11,
              color: "var(--text-secondary)",
              fontWeight: 600,
              padding: "5px 10px",
              borderRadius: 8,
              background: "var(--glass-control-bg)",
              border: "1px solid var(--chip-border)",
              boxShadow: "var(--chip-highlight)",
              whiteSpace: "nowrap",
            }}
          >
            {settings.provider?.label ?? "Refiner"}
          </span>
        </div>
      </div>
    </div>
  );
}
