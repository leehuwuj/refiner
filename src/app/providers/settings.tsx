import React from "react";
const SettingContext = React.createContext({} as AppSettings);

const SettingProvider = ({ children }: { children: React.ReactNode }) => {
  const [provider, setProvider] = React.useState<Provider>();
  const [model, setModel] = React.useState<string>();
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
