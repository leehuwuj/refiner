export interface Provider {
  name: string;
  label: string;
  models?: string[];
}

export const providerMap: Record<string, Provider> = {
  ollama: { name: "ollama", label: "Ollama", models: ["qwen3", "gemma3", "phi3", "llama3"] },
  openai: {
    name: "openai",
    label: "OpenAI",
    models: ["gpt-4.1-nano", "gpt-4.1-mini", "gpt-4o-mini"],
  },
  gemini: {
    name: "gemini",
    label: "Gemini",
    models: ["gemini-2.0-flash-lite-exp", "gemini-2.5-flash"],
  },
  groq: {
    name: "groq",
    label: "Groq",
    models: ["llama-3.1-8b-instant", "gemma2-9b-it"],
  },
};

export interface PromptSettings {
  translate?: string;
  correct?: string;
  refine?: string;
}

export type ShortcutWindowType = "popup" | "main";

export type ThemeType = "light" | "dark" | "system";

export type TextSizeType = "small" | "medium" | "large";

export interface AppSettings {
  provider?: Provider;
  model?: string;
  prompts?: PromptSettings;
  shortcutWindowType?: ShortcutWindowType;
  apiKey?: string;
  modelUrl?: string;
  thinking?: boolean;
  preferredLang?: string;
  textSize?: TextSizeType;

  setProvider: (provider: Provider) => void;
  setModel: (model: string) => void;
  setPrompts: (prompts: PromptSettings) => void;
  setShortcutWindowType: (type: ShortcutWindowType) => void;
  setModelUrl: (url: string) => void;
  setThinking: (thinking: boolean) => void;
  setPreferredLang: (lang: string) => void;
  setTextSize: (size: TextSizeType) => void;
  saveSettings: (apiKey?: string, prompts?: PromptSettings) => Promise<boolean>;
}
