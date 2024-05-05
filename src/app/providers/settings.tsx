import React from "react";
import { AppSettings, Prompts, Provider, providerMap } from "../types/settings";
const SettingContext = React.createContext({} as AppSettings);

const SettingProvider = ({ children }: { children: React.ReactNode }) => {
  const [provider, setProvider] = React.useState<Provider>(
    providerMap["openai"],
  );
  const [model, setModel] = React.useState<string>(
    provider.models?.[0] || '',
  );
  const [prompt, setPrompt] = React.useState<Prompts>();

  return (
    <SettingContext.Provider
      value={{ provider, model, prompt, setProvider, setModel, setPrompt }}
    >
      {children}
    </SettingContext.Provider>
  );
};

export { SettingContext, SettingProvider };
