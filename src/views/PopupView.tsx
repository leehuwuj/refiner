import { useContext, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { load } from "@tauri-apps/plugin-store";
import { motion, AnimatePresence } from "framer-motion";
import { Copy, Check, ExternalLink, Languages, Sparkles } from "lucide-react";

import { TranslateContext } from "@/providers/translate";
import { SettingContext } from "@/providers/settings";
import { SkeletonText } from "@/components/ui/skeleton";

type Tab = "translate" | "correct";

async function translateText(options: {
  text: string;
  provider?: string;
  model?: string;
  prompt?: string;
  preferredLang?: string;
}): Promise<string> {
  const lang = options.preferredLang || "Tiếng Việt";
  return invoke<string>("translate", {
    provider: options.provider || null,
    model: options.model || null,
    text: options.text,
    sourceLang: "auto",
    targetLang: lang,
    prompt: options.prompt || null,
  });
}

async function correctText(options: {
  text: string;
  provider?: string;
  model?: string;
  prompt?: string;
  preferredLang?: string;
}): Promise<string> {
  return invoke<string>("correct", {
    provider: options.provider || null,
    model: options.model || null,
    text: options.text,
    sourceLang: "auto",
    targetLang: "the original language of the text",
    prompt: options.prompt || null,
  });
}

async function openMainWindow() {
  try {
    const win = await WebviewWindow.getByLabel("main");
    if (win) {
      await win.show();
      await win.setFocus();
    }
  } catch (e) {
    console.error(e);
  }
}

// ── Mini Copy ─────────────────────────────────────────────────────────────────

function MiniCopy({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  return (
    <button
      onClick={copy}
      className="lg-chip"
      title={copied ? "Copied!" : "Copy"}
      style={{ width: 24, height: 24, borderRadius: 8, flexShrink: 0 }}
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
            <Check size={12} />
          </motion.div>
        ) : (
          <motion.div key="copy">
            <Copy size={12} />
          </motion.div>
        )}
      </AnimatePresence>
    </button>
  );
}

// ── Mode Tabs (icon, sliding pill — matches MainView) ───────────────────────────

const TABS: { id: Tab; icon: typeof Languages; label: string }[] = [
  { id: "translate", icon: Languages, label: "Translate" },
  { id: "correct", icon: Sparkles, label: "Correct" },
];

function ModeTabs({
  current,
  onChange,
}: {
  current: Tab;
  onChange: (tab: Tab) => void;
}) {
  return (
    <div
      role="tablist"
      style={{
        display: "flex",
        alignItems: "center",
        gap: 2,
        borderRadius: 12,
        padding: 3,
        background: "var(--glass-control-bg)",
        border: "1px solid var(--chip-border)",
        boxShadow: "var(--chip-highlight)",
      }}
    >
      {TABS.map(({ id, icon: Icon, label }) => {
        const active = id === current;
        return (
          <button
            key={id}
            role="tab"
            aria-selected={active}
            title={label}
            onClick={() => onChange(id)}
            onKeyDown={(e) => {
              if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
                onChange(id === "translate" ? "correct" : "translate");
              }
            }}
            style={{
              position: "relative",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              padding: "5px 12px",
              borderRadius: 8,
              border: "none",
              cursor: "pointer",
              background: "transparent",
              color: active ? "var(--text-primary)" : "var(--text-tertiary)",
              transition: "color 0.2s",
            }}
          >
            {active && (
              <motion.div
                layoutId="popup-mode-pill"
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
            <Icon size={15} style={{ position: "relative", zIndex: 1 }} />
          </button>
        );
      })}
    </div>
  );
}

// ── Result Panel ────────────────────────────────────────────────────────────────

function ResultPanel({
  label,
  content,
  loading,
}: {
  label: string;
  content: string;
  loading: boolean;
}) {
  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        minHeight: 0,
        margin: "0 10px 8px",
        borderRadius: 14,
        background:
          "linear-gradient(180deg, var(--glass-panel-bg) 0%, transparent 100%)",
        border: "1px solid var(--glass-border)",
        boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06)",
      }}
    >
      {/* Panel header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "8px 10px 4px",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: 9,
            fontWeight: 700,
            letterSpacing: "0.07em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
          }}
        >
          {label}
        </span>
        {content && <MiniCopy text={content} />}
      </div>

      {/* Panel body */}
      <div
        style={{ flex: 1, overflowY: "auto", padding: "0 12px 10px" }}
        className="scrollbar-hide"
      >
        <AnimatePresence mode="wait">
          {loading ? (
            <motion.div
              key="skeleton"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            >
              <SkeletonText lines={3} />
            </motion.div>
          ) : content ? (
            <motion.p
              key="content"
              initial={{ opacity: 0, y: 2 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.15 }}
              style={{
                fontSize: 13,
                fontWeight: 600,
                lineHeight: 1.55,
                color: "var(--text-primary)",
                margin: 0,
              }}
            >
              {content}
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
                  fontWeight: 500,
                  color: "var(--text-tertiary)",
                  margin: 0,
                }}
              >
                Waiting for selection…
              </p>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

// ── Popup View ────────────────────────────────────────────────────────────────

export default function PopupView() {
  const ctx = useContext(TranslateContext);
  const settings = useContext(SettingContext);
  const [activeTab, setActiveTab] = useState<Tab>("translate");
  const isMounted = useRef(false);

  const ctxRef = useRef(ctx);
  const settingsRef = useRef(settings);
  useEffect(() => {
    ctxRef.current = ctx;
  }, [ctx]);
  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  const runTranslation = useRef((text: string) => {
    const { provider, model, prompts, preferredLang } = settingsRef.current;
    ctxRef.current.setTranslating(true);
    ctxRef.current.changeResult({ translate: "", correct: "" });

    translateText({
      text,
      provider: provider?.name,
      model,
      prompt: prompts?.translate,
      preferredLang,
    })
      .then((translation) => {
        ctxRef.current.changeResult({ translate: translation, correct: "" });
        return correctText({
          text,
          provider: provider?.name,
          model,
          prompt: prompts?.correct,
          preferredLang,
        }).then((correction) => {
          ctxRef.current.changeResult({
            translate: translation,
            correct: correction,
          });
        });
      })
      .catch((e) => {
        console.error(e);
        ctxRef.current.changeResult({
          translate: "Translation failed",
          correct: "Correction failed",
        });
      })
      .finally(() => {
        ctxRef.current.setTranslating(false);
      });
  });

  useEffect(() => {
    if (isMounted.current) return;
    isMounted.current = true;

    // Listen for events when popup is already open and shortcut fires again
    listen("shortcut-popup-translate", (event) => {
      let raw = event.payload as string;
      if (raw.includes("text:")) raw = raw.split("text:")[1].trim();
      raw = raw.replace(/^"|"$/g, "").trim();
      if (raw) runTranslation.current(raw);
    });

    // On first mount, read pending text from store (handles race condition
    // where the event was emitted before this JS loaded)
    load("store.bin", { autoSave: false })
      .then((store) => {
        store.get<string>("POPUP_PENDING_TEXT").then((text) => {
          if (text) {
            store.delete("POPUP_PENDING_TEXT");
            store.save();
            runTranslation.current(text);
          }
        });
      })
      .catch(console.error);
  }, []);

  const translateLoading = !!ctx.translating && !ctx.result?.translate;
  const correctLoading = !!ctx.translating && !ctx.result?.correct;

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
      {/* ── Glass surface ── */}
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
          position: "relative",
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
            left: "10%",
            right: "10%",
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
            padding: "8px 10px 6px",
            flexShrink: 0,
            position: "relative",
            zIndex: 2,
          }}
        >
          <ModeTabs current={activeTab} onChange={setActiveTab} />

          <button
            onClick={openMainWindow}
            title="Open Refiner"
            className="lg-chip"
            style={{ width: 28, height: 28, borderRadius: "50%", flexShrink: 0 }}
          >
            <ExternalLink size={13} />
          </button>
        </div>

        {/* ── Content ── */}
        <div
          style={{
            flex: 1,
            display: "flex",
            minHeight: 0,
            position: "relative",
            zIndex: 2,
          }}
        >
          <AnimatePresence mode="wait">
            {activeTab === "translate" ? (
              <motion.div
                key="translate"
                initial={{ opacity: 0, x: -8 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 8 }}
                transition={{ duration: 0.18 }}
                style={{ flex: 1, display: "flex", minHeight: 0 }}
              >
                <ResultPanel
                  label="Translation"
                  content={ctx.result?.translate || ""}
                  loading={translateLoading}
                />
              </motion.div>
            ) : (
              <motion.div
                key="correct"
                initial={{ opacity: 0, x: 8 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -8 }}
                transition={{ duration: 0.18 }}
                style={{ flex: 1, display: "flex", minHeight: 0 }}
              >
                <ResultPanel
                  label="Correction"
                  content={ctx.result?.correct || ""}
                  loading={correctLoading}
                />
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
