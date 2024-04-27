import React from "react";
const SettingContext = React.createContext({} as AppSettings);

const SettingProvider = ({ children }: { children: React.ReactNode }) => {
    const [settings, setSettings] = React.useState<AppSettings>({
        provider: { name: "ollama", label: "Ollama" },
        model: "phi3",
    });

    return (
        <SettingContext.Provider value={{ settings, setSettings } as AppSettings}>
            {children}
        </SettingContext.Provider>
    );
};

export { SettingContext, SettingProvider };