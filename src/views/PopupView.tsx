import { cn } from "@/lib/utils";
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
      className="hover:text-[var(--text-primary)]"
      style={{
        padding: "3px 5px",
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
      <AnimatePresence mode="wait">
        {copied ? (
          <motion.div key="check" initial={{ scale: 0 }} animate={{ scale: 1 }}>
            <Check size={10} />
          </motion.div>
        ) : (
          <motion.div key="copy">
            <Copy size={10} />
          </motion.div>
        )}
      </AnimatePresence>
    </button>
  );
}

// ── Section ───────────────────────────────────────────────────────────────────

function Section({
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
        padding: "8px 12px 6px",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: 4,
        }}
      >
        <span
          style={{
            fontSize: 9,
            fontWeight: 600,
            letterSpacing: "0.06em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
          }}
        >
          {label}
        </span>
        {content && <MiniCopy text={content} />}
      </div>

      <div style={{ flex: 1, overflowY: "auto" }} className="scrollbar-hide">
        <AnimatePresence mode="wait">
          {loading ? (
            <motion.div
              key="skeleton"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            >
              <SkeletonText lines={2} />
            </motion.div>
          ) : content ? (
            <motion.p
              key="content"
              initial={{ opacity: 0, y: 2 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.15 }}
              style={{
                fontSize: 11,
                lineHeight: 1.6,
                color: "var(--text-primary)",
                margin: 0,
              }}
            >
              {content}
            </motion.p>
          ) : (
            <motion.p
              key="empty"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              style={{ fontSize: 10, color: "var(--text-tertiary)", margin: 0 }}
            >
              —
            </motion.p>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

// ── Mode Switcher ─────────────────────────────────────────────────────────────
function ModeSwitcher({
  activeTab,
  setActiveTab,
}: {
  activeTab: "translate" | "correct";
  setActiveTab: (tab: "translate" | "correct") => void;
}) {
  return (
    <div role="tablist" className="flex justify-center p-[8px_4px_4px] relative z-10">
      <div className="flex bg-[var(--glass-control-bg)] rounded-lg p-[3px] border border-[var(--glass-border)] backdrop-blur-[10px]">
        <button
          role="tab"
          aria-selected={activeTab === "translate"}
          onClick={() => setActiveTab("translate")}
          onKeyDown={(e) => {
            if (e.key === 'ArrowRight') setActiveTab('correct');
            if (e.key === 'ArrowLeft') setActiveTab('translate');
          }}
          className={cn(
            "p-[4px_8px] rounded-md border-none transition-all duration-200 flex items-center",
            activeTab === "translate" 
              ? "bg-[var(--glass-control-hover)] text-white" 
              : "text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
          )}
        >
          <Languages size={16} />
        </button>
        <button
          role="tab"
          aria-selected={activeTab === "correct"}
          onClick={() => setActiveTab("correct")}
          onKeyDown={(e) => {
            if (e.key === 'ArrowRight') setActiveTab('translate');
            if (e.key === 'ArrowLeft') setActiveTab('correct');
          }}
          className={cn(
            "p-[4px_8px] rounded-md border-none transition-all duration-200 flex items-center",
            activeTab === "correct" 
              ? "bg-[var(--glass-control-hover)] text-white" 
              : "text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
          )}
        >
          <Sparkles size={16} />
        </button>
      </div>
    </div>
  );
}

// ── Popup View ────────────────────────────────────────────────────────────────

export default function PopupView() {
  const ctx = useContext(TranslateContext);
  const settings = useContext(SettingContext);
  const [activeTab, setActiveTab] = useState<'translate' | 'correct'>('translate');
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
    <div style={{ width: "100%", height: "100%", background: "transparent", borderRadius: "var(--radius-app)", overflow: "hidden" }}>
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
        {/* Noise */}
        <div style={{ position: "absolute", inset: 0, backgroundImage: "var(--noise-url)", opacity: 0.03, pointerEvents: "none", zIndex: 0 }} />
        {/* Specular highlight */}
        <div style={{ position: "absolute", top: 0, left: "12%", right: "12%", height: 1, background: "linear-gradient(90deg, transparent, rgba(255,255,255,0.10), transparent)", pointerEvents: "none", zIndex: 1 }} />

        {/* Drag handle */}
        <div data-tauri-drag-region style={{ height: 6, flexShrink: 0, cursor: "grab", position: "relative", zIndex: 2 }} />

        {/* Mode Switcher */}
        <div style={{ display: 'flex', justifyContent: 'center', padding: '8px 4px 4px', position: 'relative', zIndex: 10 }}>
          <div style={{ display: 'flex', background: 'rgba(255, 255, 255, 0.05)', borderRadius: '8px', padding: '3px', border: '1px solid rgba(255, 255, 255, 0.1)', backdropFilter: 'blur(10px)' }}>
            <button
              onClick={() => setActiveTab('translate')}
              style={{
                padding: '4px 8px',
                borderRadius: '6px',
                border: 'none',
                background: activeTab === 'translate' ? 'rgba(255, 255, 255, 0.1)' : 'transparent',
                color: activeTab === 'translate' ? '#fff' : 'var(--text-tertiary)',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                transition: 'all 0.2s ease',
              }}
            >
              <Languages size={16} />
            </button>
            <button
              onClick={() => setActiveTab('correct')}
              style={{
                padding: '4px 8px',
                borderRadius: '6px',
                border: 'none',
                background: activeTab === 'correct' ? 'rgba(255, 255, 255, 0.1)' : 'transparent',
                color: activeTab === 'correct' ? '#fff' : 'var(--text-tertiary)',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                transition: 'all 0.2s ease',
              }}
            >
              <Sparkles size={16} />
            </button>
          </div>
        </div>

        {/* Content Area */}
        <div style={{ flex: 1, overflowY: "auto", position: "relative", zIndex: 2 }} className="scrollbar-hide">
          <AnimatePresence mode="wait">
            {activeTab === 'translate' ? (
              <motion.div
                key="translate"
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 10 }}
                transition={{ duration: 0.2 }}
              >
                <Section label="Translation" content={ctx.result?.translate || ""} loading={translateLoading} />
              </motion.div>
            ) : (
              <motion.div
                key="correct"
                initial={{ opacity: 0, x: 10 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -10 }}
                transition={{ duration: 0.2 }}
              >
                <Section label="Correction" content={ctx.result?.correct || ""} loading={correctLoading} />
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        {/* Footer */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", padding: "2px 10px 6px", position: "relative", zIndex: 2 }}>
          <button
            onClick={openMainWindow}
            title="Open Refiner"
            className="hover:bg-[var(--glass-control-bg)] hover:text-[var(--text-primary)]"
            style={{ padding: "3px 5px", borderRadius: 6, border: "none", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer", display: "flex", alignItems: "center", transition: "all 0.15s" }}
          >
            <ExternalLink size={10} />
          </button>
        </div>
      </div>
    </div>
  );
}
