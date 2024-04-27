interface Provider {
  name: string;
  label: string;
  models?: string[];
}

const providers: Provider[] = [
  { name: "ollama", label: "Ollama" },
  { name: "openai", label: "OpenAI" },
];

type promptTypes = ["translate", "refine", "refineFormal"];

interface Prompts {
  type: promptTypes;
  value?: string;
}

interface AppSettings {
  provider?: Provider;
  model?: string;
  prompt?: Prompts;

  setProvider: (provider: Provider) => void;
  setModel: (model: string) => void;
  setPrompt: (prompt: Prompts) => void;
}
