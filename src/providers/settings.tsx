import React from "react";
import type {
  AppSettings,
  Provider,
  PromptSettings,
  ShortcutWindowType,
  TextSizeType,
} from "@/types/settings";
import { providerMap } from "@/types/settings";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";

const SettingContext = React.createContext({} as AppSettings);

const SettingProvider = ({ children }: { children: React.ReactNode }) => {
  const [provider, setProvider] = React.useState<Provider>(
    providerMap["openai"],
  );
  const [model, setModel] = React.useState<string>(
    providerMap["openai"].models?.[0] || "",
  );
  const [prompts, setPrompts] = React.useState<PromptSettings>({});
  const [shortcutWindowType, setShortcutWindowType] =
    React.useState<ShortcutWindowType>("main");
  const [apiKey, setApiKey] = React.useState<string>("");
  const [ollamaEndpoint, setOllamaEndpoint] = React.useState<string>(
    "http://localhost:11434",
  );
  const [ollamaThinking, setOllamaThinking] = React.useState<boolean>(true);
  const [preferredLang, setPreferredLang] =
    React.useState<string>("Tiếng Việt");
  const [textSize, setTextSize] = React.useState<TextSizeType>("medium");

  React.useEffect(() => {
    const loadSettings = async () => {
      try {
        const saved = await invoke<{
          provider: string | null;
          model: string | null;
          api_key: string | null;
          shortcut_window_type: string | null;
          ollama_endpoint: string | null;
          ollama_thinking: boolean | null;
          prompt_translate: string | null;
          prompt_correct: string | null;
          prompt_refine: string | null;
          preferred_lang: string | null;
          text_size: string | null;
        }>("get_settings");

        let loadedProvider = provider;
        if (saved.provider) {
          const foundProvider = providerMap[saved.provider];
          if (foundProvider) {
            loadedProvider = foundProvider;
            setProvider(foundProvider);
          }
        }

        if (saved.model) {
          setModel(saved.model);
        } else if (loadedProvider.models && loadedProvider.models.length > 0) {
          setModel(loadedProvider.models[0]);
        }

        if (saved.shortcut_window_type) {
          setShortcutWindowType(
            saved.shortcut_window_type as ShortcutWindowType,
          );
        }

        if (saved.api_key) {
          setApiKey(saved.api_key);
        }

        if (saved.ollama_endpoint) {
          setOllamaEndpoint(saved.ollama_endpoint);
        }

        if (
          saved.ollama_thinking !== null &&
          saved.ollama_thinking !== undefined
        ) {
          setOllamaThinking(saved.ollama_thinking);
        }

        setPrompts({
          translate: saved.prompt_translate ?? undefined,
          correct: saved.prompt_correct ?? undefined,
          refine: saved.prompt_refine ?? undefined,
        });

        if (saved.preferred_lang) {
          setPreferredLang(saved.preferred_lang);
        }

        if (saved.text_size) {
          setTextSize(saved.text_size as TextSizeType);
        }
      } catch (error) {
        console.error("Failed to load settings:", error);
      }
    };
    loadSettings();
  }, []);

  React.useEffect(() => {
    // Use the webview's native page zoom rather than CSS `zoom` on <html>.
    // CSS zoom breaks viewport units (100vh/100vw) and the coordinate math
    // that portalled popovers (Radix Select) rely on, causing overflow and
    // mispositioned dropdowns. Native zoom adjusts the viewport correctly.
    const scaleMap: Record<TextSizeType, number> = {
      small: 0.9,
      medium: 1,
      large: 1.1,
    };
    const factor = scaleMap[textSize] ?? 1;
    getCurrentWebview()
      .setZoom(factor)
      .catch((err) => console.error("Failed to set webview zoom:", err));
  }, [textSize]);

  const saveSettings = async (
    inputApiKey?: string,
    promptsOverride?: PromptSettings,
  ) => {
    const promptsToSave = promptsOverride ?? prompts;
    try {
      await invoke("save_settings", {
        apiKey: inputApiKey || null,
        shortcutWindowType: shortcutWindowType,
        provider: provider.name,
        model: model,
        ollamaEndpoint: ollamaEndpoint ?? null,
        ollamaThinking: ollamaThinking,
        promptTranslate: promptsToSave.translate ?? "",
        promptCorrect: promptsToSave.correct ?? "",
        promptRefine: promptsToSave.refine ?? "",
        preferredLang: preferredLang,
        textSize: textSize,
      });

      if (inputApiKey) {
        setApiKey(inputApiKey);
      }

      return true;
    } catch (error) {
      console.error("Failed to save settings:", error);
      return false;
    }
  };

  return (
    <SettingContext.Provider
      value={{
        provider,
        model,
        prompts,
        shortcutWindowType,
        apiKey,
        ollamaEndpoint,
        ollamaThinking,
        preferredLang,
        setProvider,
        setModel,
        setPrompts,
        setShortcutWindowType,
        setOllamaEndpoint,
        setOllamaThinking,
        setPreferredLang,
        textSize,
        setTextSize,
        saveSettings,
      }}
    >
      {children}
    </SettingContext.Provider>
  );
};

export { SettingContext, SettingProvider };
