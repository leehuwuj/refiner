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

  return (
    <TranslateContext.Provider
      value={{
        languageConfig,
        inputText,
        result,
        translating,
        changeLangConfig: (lang: Language) => setLanguageConfig(lang),
        changeInputText: (text: string) => setInputText(text),
        changeResult: (res: ResultTexts) => setResult(res),
        setTranslating: (isTranslating: boolean) =>
          setTranslating(isTranslating),
      }}
    >
      {children}
    </TranslateContext.Provider>
  );
};

export { TranslateContext, TranslateProvider };
