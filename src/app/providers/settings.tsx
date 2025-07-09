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

  // Load settings from storage on component mount
  React.useEffect(() => {
    const loadSettings = async () => {
      try {
        const store = await load("store.bin");
        const savedShortcutWindowType = await store.get("SHORTCUT_WINDOW_TYPE");
        if (savedShortcutWindowType !== null) {
          setShortcutWindowType(savedShortcutWindowType as ShortcutWindowType);
        }
      } catch (error) {
        console.log("Failed to load settings:", error);
        // Keep default values if loading fails
      }
    };
    loadSettings();
  }, []);

  const saveSettings = async (apiKey?: string) => {
    try {
      await invoke("save_settings", {
        apiKey: apiKey || null,
        shortcutWindowType: shortcutWindowType
      });

      return true;
    } catch (error) {
      console.error("Failed to save settings:", error);
      return false;
    }
  };

  return (
    <SettingContext.Provider
      value={{ provider, model, prompt, shortcutWindowType, setProvider, setModel, setPrompt, setShortcutWindowType, saveSettings }}
    >
      {children}
    </SettingContext.Provider>
  );
};

export { SettingContext, SettingProvider };
