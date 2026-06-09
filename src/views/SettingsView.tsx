import React, { useContext, useState, useEffect } from "react";
import { Save, Bot, Key, Zap, Monitor, Palette, Check, AlertCircle, MessageSquare, Languages } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

import { SettingContext } from "@/providers/settings";
import { providerMap } from "@/types/settings";
import type { ShortcutWindowType, TextSizeType } from "@/types/settings";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTheme } from "@/providers/theme";
import type { ThemeType } from "@/types/settings";
import { cn } from "@/lib/utils";

// ── Section wrapper ───────────────────────────────────────────────────────────

function Section({
  icon: Icon,
  title,
  children,
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

// ── Row (label + control) ─────────────────────────────────────────────────────

function Row({ label, description, children }: { label: string; description?: string; children: React.ReactNode }) {
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

// ── Settings View ─────────────────────────────────────────────────────────────

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
    "You are an expert editor. Rewrite the following text in a more conversational style, in {target_lang}. Provide only the rewritten text, without any additional explanations or formatting.",
};

export default function SettingsView() {
  const s = useContext(SettingContext);
  const { theme, setTheme } = useTheme();
  const [localApiKey, setLocalApiKey] = useState("");
  const [localPrompts, setLocalPrompts] = useState({ translate: "", correct: "", refine: "" });
  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "success" | "error">("idle");

  const isOllama = s.provider?.name === "ollama";

  useEffect(() => {
    if (s.apiKey) setLocalApiKey(s.apiKey);
  }, [s.apiKey]);

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
    } catch {
      setSaveStatus("error");
    } finally {
      setIsSaving(false);
      setTimeout(() => setSaveStatus("idle"), 3000);
    }
  };

  return (
    <div className="min-h-screen bg-[var(--surface-bg)] text-[var(--text-primary)]">
      {/* Header */}
      <div className="sticky top-0 z-10 flex items-center justify-between px-5 py-4 border-b border-[var(--glass-border)] bg-[var(--surface-bg)]">
        <div>
          <h1 className="text-base font-semibold">Settings</h1>
          <p className="text-[11px] text-[var(--text-tertiary)]">Configure Refiner</p>
        </div>
        <Button
          onClick={handleSave}
          disabled={isSaving}
          size="sm"
          className="gap-1.5"
        >
          <AnimatePresence mode="wait">
            {saveStatus === "success" ? (
              <motion.div key="check" initial={{ scale: 0 }} animate={{ scale: 1 }} className="flex items-center gap-1.5">
                <Check size={13} /> Saved
              </motion.div>
            ) : (
              <motion.div key="save" className="flex items-center gap-1.5">
                {isSaving ? (
                  <div className="h-3 w-3 rounded-full border-2 border-white/40 border-t-white animate-spin" />
                ) : (
                  <Save size={13} />
                )}
                {isSaving ? "Saving..." : "Save"}
              </motion.div>
            )}
          </AnimatePresence>
        </Button>
      </div>

      {/* Body */}
      <div className="p-5 space-y-4 overflow-y-auto">
        {/* Provider & Model */}
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
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="Select provider" />
              </SelectTrigger>
              <SelectContent>
                {Object.values(providerMap).map((p) => (
                  <SelectItem key={p.name} value={p.name}>
                    {p.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Row>

          {isOllama ? (
            <Input
              label="Model"
              value={s.model ?? ""}
              onChange={(e) => s.setModel(e.target.value)}
              placeholder="e.g. gemma3, llama3:8b"
              description={`Suggestions: ${providerMap["ollama"].models?.join(", ")}`}
            />
          ) : (
            <Row label="Model">
              <Select value={s.model} onValueChange={(m) => s.setModel(m)}>
                <SelectTrigger className="w-[180px]">
                  <SelectValue placeholder="Select model" />
                </SelectTrigger>
                <SelectContent>
                  {s.provider?.models?.map((m) => (
                    <SelectItem key={m} value={m}>
                      {m}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </Row>
          )}
        </Section>

        {/* API / Endpoint */}
        <Section icon={Key} title="Authentication">
          {isOllama ? (
            <Input
              label="Ollama Endpoint"
              value={s.ollamaEndpoint ?? "http://localhost:11434"}
              onChange={(e) => s.setOllamaEndpoint(e.target.value)}
              placeholder="http://localhost:11434"
            />
          ) : (
            <Input
              label="API Key"
              type="password"
              value={localApiKey}
              onChange={(e) => setLocalApiKey(e.target.value)}
              placeholder="sk-..."
            />
          )}
        </Section>

        {/* Ollama-specific */}
        {isOllama && (
          <Section icon={Zap} title="Inference">
            <Row
              label="Enable Thinking"
              description="Extended reasoning for models like qwen3"
            >
              <Switch
                checked={s.ollamaThinking ?? true}
                onCheckedChange={(v) => s.setOllamaThinking(v)}
              />
            </Row>
          </Section>
        )}

        {/* Shortcuts */}
        <Section icon={Monitor} title="Shortcut Behavior">
          <Row label="Window Type" description="Which window opens on shortcut trigger">
            <Select
              value={s.shortcutWindowType}
              onValueChange={(v) => s.setShortcutWindowType(v as ShortcutWindowType)}
            >
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="Select" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="popup">Small Popup</SelectItem>
                <SelectItem value="main">Main Window</SelectItem>
              </SelectContent>
            </Select>
          </Row>
        </Section>

        {/* Language */}
        <Section icon={Languages} title="Language">
          <Row label="Preferred Language" description="Default target language for translations">
            <Select
              value={s.preferredLang ?? "Tiếng Việt"}
              onValueChange={(v) => s.setPreferredLang(v)}
            >
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="Select language" />
              </SelectTrigger>
              <SelectContent>
                {LANGUAGES.map((l) => (
                  <SelectItem key={l.value} value={l.label}>
                    {l.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Row>
        </Section>

        {/* Prompts */}
        <Section icon={MessageSquare} title="Prompts">
          <p className="text-[11px] text-[var(--text-tertiary)] -mt-1">
            Available variables: <code className="text-[var(--text-secondary)]">{"{original_lang}"}</code>, <code className="text-[var(--text-secondary)]">{"{target_lang}"}</code>. Leave blank to use defaults.
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
                  "focus:outline-none focus:border-[var(--glass-border-bright)]",
                  "scrollbar-hide transition-colors",
                )}
              />
            </div>
          ))}
        </Section>

        {/* Appearance */}
        <Section icon={Palette} title="Appearance">
          <Row label="Theme">
            <Select
              value={theme}
              onValueChange={(v) => setTheme(v as ThemeType)}
            >
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="Select theme" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">System</SelectItem>
                <SelectItem value="dark">Dark</SelectItem>
                <SelectItem value="light">Light</SelectItem>
              </SelectContent>
            </Select>
          </Row>
          <Row label="Text Size" description="Scales all text across windows">
            <Select
              value={s.textSize ?? "medium"}
              onValueChange={(v) => s.setTextSize(v as TextSizeType)}
            >
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="Select size" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="small">Small</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="large">Large</SelectItem>
              </SelectContent>
            </Select>
          </Row>
        </Section>

        {/* Status message */}
        <AnimatePresence>
          {saveStatus === "error" && (
            <motion.div
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              className="flex items-center gap-2 rounded-lg px-3 py-2 bg-red-500/10 border border-red-500/25 text-red-400 text-sm"
            >
              <AlertCircle size={14} />
              Failed to save settings. Please try again.
            </motion.div>
          )}
        </AnimatePresence>

        {/* Footer */}
        <p className="text-center text-[10px] text-[var(--text-tertiary)] pt-2">
          © 2026 Refiner App
        </p>
      </div>
    </div>
  );
}
