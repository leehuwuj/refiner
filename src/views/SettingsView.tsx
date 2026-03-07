import React, { useContext, useState, useEffect } from "react";
import { Save, Bot, Key, Zap, Monitor, Palette, Check, AlertCircle } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

import { SettingContext } from "@/providers/settings";
import { providerMap } from "@/types/settings";
import type { ShortcutWindowType } from "@/types/settings";
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

export default function SettingsView() {
  const s = useContext(SettingContext);
  const { theme, setTheme } = useTheme();
  const [localApiKey, setLocalApiKey] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "success" | "error">("idle");

  const isOllama = s.provider?.name === "ollama";

  useEffect(() => {
    if (s.apiKey) setLocalApiKey(s.apiKey);
  }, [s.apiKey]);

  const handleSave = async () => {
    setIsSaving(true);
    setSaveStatus("idle");
    try {
      const ok = await s.saveSettings(localApiKey || undefined);
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
          © 2025 Refiner App
        </p>
      </div>
    </div>
  );
}
