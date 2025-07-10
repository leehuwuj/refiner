import React from "react";
import { AppSettings, Prompts, Provider, providerMap, ShortcutWindowType } from "../types/settings";
import { load } from "@tauri-apps/plugin-store";
import { invoke } from "@tauri-apps/api/core";

const SettingContext = React.createContext({} as AppSettings);

const SettingProvider = ({ children }: { children: React.ReactNode }) => {
  const [provider, setProvider] = React.useState<Provider>(
    providerMap["openai"],
  );
  const [model, setModel] = React.useState<string>(
    provider.models?.[0] || '',
  );
  const [prompt, setPrompt] = React.useState<Prompts>();
  const [shortcutWindowType, setShortcutWindowType] = React.useState<ShortcutWindowType>("main");
  const [apiKey, setApiKey] = React.useState<string>("");

  const clearInvalidPrompt = async () => {
    try {
      const store = await load("store.bin");
      store.delete("PROMPT");
      await store.save();
    } catch (error) {
      console.error("Failed to clear invalid prompt:", error);
    }
  };

  // Load settings from storage on component mount
  React.useEffect(() => {
    const loadSettings = async () => {
      try {
        const store = await load("store.bin");

        // Load provider first
        const savedProvider = await store.get("PROVIDER");
        let loadedProvider = provider; // Default to current provider
        if (savedProvider !== null) {
          const providerName = savedProvider as string;
          const foundProvider = providerMap[providerName];
          if (foundProvider) {
            loadedProvider = foundProvider;
            setProvider(foundProvider);
          }
        }

        // Load model (with fallback to provider's first model)
        const savedModel = await store.get("MODEL");
        if (savedModel !== null) {
          const modelName = savedModel as string;
          // Verify the model exists in the loaded provider
          if (loadedProvider.models && loadedProvider.models.includes(modelName)) {
            setModel(modelName);
          } else if (loadedProvider.models && loadedProvider.models.length > 0) {
            // Fallback to first model if saved model doesn't exist
            setModel(loadedProvider.models[0]);
          }
        } else if (loadedProvider.models && loadedProvider.models.length > 0) {
          // No saved model, use first model of the provider
          setModel(loadedProvider.models[0]);
        }

        // Load shortcut window type
        const savedShortcutWindowType = await store.get("SHORTCUT_WINDOW_TYPE");
        if (savedShortcutWindowType !== null) {
          setShortcutWindowType(savedShortcutWindowType as ShortcutWindowType);
        }

        // Load API key
        const savedApiKey = await store.get("OPENAI_API_KEY");
        if (savedApiKey !== null) {
          setApiKey(savedApiKey as string);
        }

        // Load prompt settings
        const savedPrompt = await store.get("PROMPT");
        if (savedPrompt !== null && savedPrompt !== "undefined" && savedPrompt !== undefined) {
          try {
            const promptString = savedPrompt as string;
            if (promptString && promptString !== "undefined") {
              setPrompt(JSON.parse(promptString));
            }
          } catch (error) {
            console.error("Failed to parse saved prompt:", error);
            // Clear invalid prompt data
            clearInvalidPrompt();
          }
        }
      } catch (error) {
        console.log("Failed to load settings:", error);
        // Keep default values if loading fails
      }
    };
    loadSettings();
  }, []);

  const saveSettings = async (inputApiKey?: string) => {
    try {
      // Only stringify prompt if it's a valid object
      let promptToSave = null;
      if (prompt && typeof prompt === 'object') {
        promptToSave = JSON.stringify(prompt);
      }

      await invoke("save_settings", {
        apiKey: inputApiKey || null,
        shortcutWindowType: shortcutWindowType,
        provider: provider.name,
        model: model,
        prompt: promptToSave
      });

      // Update local API key state if provided
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
      value={{ provider, model, prompt, shortcutWindowType, apiKey, setProvider, setModel, setPrompt, setShortcutWindowType, saveSettings }}
    >
      {children}
    </SettingContext.Provider>
  );
};

export { SettingContext, SettingProvider };
