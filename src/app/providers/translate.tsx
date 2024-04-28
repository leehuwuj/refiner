import React from "react";

const TranslateContext = React.createContext({} as TranslateContextType);

const TranslateProvider = ({ children }: { children: React.ReactNode }) => {
  const [languageConfig, setLanguageConfig] = React.useState<Language>({
    sourceLang: "en",
    targetLang: "vi",
  });
  const [inputText, setInputText] = React.useState<string>("");
  const [result, setResult] = React.useState<ResultTexts>({});
  const [translating, setTranslating] = React.useState<boolean>(false);
  const [currentMode, setCurrentMode] = React.useState<Mode>(
    "Translate" as Mode,
  );

  return (
    <TranslateContext.Provider
      value={{
        languageConfig,
        inputText,
        result,
        translating,
        currentMode,
        changeLangConfig: (lang: Language) => setLanguageConfig(lang),
        changeInputText: (text: string) => setInputText(text),
        changeResult: (res: ResultTexts) => setResult(res),
        setTranslating: (isTranslating: boolean) =>
          setTranslating(isTranslating),
        setCurrentMode: (mode: Mode) => setCurrentMode(mode),
      }}
    >
      {children}
    </TranslateContext.Provider>
  );
};

export { TranslateContext, TranslateProvider };
