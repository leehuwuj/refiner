export interface Provider {
  name: string;
  label: string;
  models?: string[];
}

export const providerMap: Record<string, Provider> = {
  ollama: { name: "ollama", label: "Ollama", models: ["phi3", "llama3"] },
  openai: {
    name: "openai",
    label: "OpenAI",
    models: ["gpt-4o-mini", "gpt-4.1-nano", "gpt-4.1-mini"],
  },
};

export type promptTypes = ["translate", "correct", "refine"];

export interface Prompts {
  type: promptTypes;
  value?: string;
}

export interface AppSettings {
  provider?: Provider;
  model?: string;
  prompt?: Prompts;
  doubleClickEnabled?: boolean;

  setProvider: (provider: Provider) => void;
  setModel: (model: string) => void;
  setPrompt: (prompt: Prompts) => void;
  setDoubleClickEnabled: (enabled: boolean) => void;
  saveSettings: (apiKey?: string) => Promise<boolean>;
}
