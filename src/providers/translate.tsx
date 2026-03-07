import React from "react";
import type { TranslateContextType, LanguageConfig, ResultTexts, Mode } from "@/types/translate";

const TranslateContext = React.createContext({} as TranslateContextType);

const TranslateProvider = ({ children }: { children: React.ReactNode }) => {
  const [languageConfig, setLanguageConfig] = React.useState<LanguageConfig>({
    sourceLang: { code: "en", label: "English" },
    targetLang: { code: "vn", label: "Tiếng Việt" },
  });
  const [inputText, setInputText] = React.useState<string>("");
  const [result, setResult] = React.useState<ResultTexts>({});
  const [translating, setTranslating] = React.useState<boolean>(false);
  const [currentMode, setCurrentMode] = React.useState<Mode>("Translate");

  return (
    <TranslateContext.Provider
      value={{
        languageConfig,
        inputText,
        result,
        translating,
        currentMode,
        changeLangConfig: (lang: LanguageConfig) => setLanguageConfig(lang),
        changeInputText: (text: string) => setInputText(text),
        changeResult: (res: ResultTexts) => setResult((prev) => ({ ...prev, ...res })),
        setTranslating: (isTranslating: boolean) => setTranslating(isTranslating),
        setCurrentMode: (mode: Mode) => setCurrentMode(mode),
      }}
    >
      {children}
    </TranslateContext.Provider>
  );
};

export { TranslateContext, TranslateProvider };
