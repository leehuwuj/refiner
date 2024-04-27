interface Language {
  code: string;
  label: string;
}

type LanguageConfig = {
  sourceLang: Language;
  targetLang: Language;
};

interface ResultTexts {
  translate?: string;
  correct?: string;
  refine?: string;
}

type Mode = "Translate" | "Correct" | "Refine";

interface TranslateContextType {
  languageConfig: LanguageConfig;
  inputText?: string;
  result?: ResultTexts;
  translating?: boolean;
  currentMode: Mode;

  changeLangConfig: (lang: LanguageConfig) => void;
  changeInputText: (text: string) => void;
  changeResult: (result: ResultTexts) => void;
  setTranslating: (isTranslating: boolean) => void;
  setCurrentMode: (mode: Mode) => void;
}
