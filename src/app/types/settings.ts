export interface Provider {
  name: string;
  label: string;
  models?: string[];
}

export const providerMap: Record<string, Provider> = {
  ollama: { name: "ollama", label: "Ollama", models: ["gemma3", "phi3", "llama3"] },
  openai: {
    name: "openai",
    label: "OpenAI",
    models: ["gpt-4.1-nano", "gpt-4.1-mini", "gpt-4o-mini"],
  },
  gemini: {
    name: "gemini",
    label: "Gemini",
    models: ["gemini-2.0-flash-lite", "gemini-2.5-flash"],
  },
};

export type promptTypes = ["translate", "correct", "refine"];

export interface Prompts {
  type: promptTypes;
  value?: string;
}

export type ShortcutWindowType = "popup" | "main";

export type ThemeType = "light" | "dark" | "system";

export interface AppSettings {
  provider?: Provider;
  model?: string;
  prompt?: Prompts;
  shortcutWindowType?: ShortcutWindowType;
  apiKey?: string;

  setProvider: (provider: Provider) => void;
  setModel: (model: string) => void;
  setPrompt: (prompt: Prompts) => void;
  setShortcutWindowType: (type: ShortcutWindowType) => void;
  saveSettings: (apiKey?: string) => Promise<boolean>;
}
