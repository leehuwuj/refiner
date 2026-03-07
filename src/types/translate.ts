export interface Language {
  code: string;
  label: string;
}

export type LanguageConfig = {
  sourceLang: Language;
  targetLang: Language;
};

export interface ResultTexts {
  translate?: string;
  correct?: string;
  refine?: string;
}

export type Mode = "Translate" | "Correct" | "Refine";

export interface TranslateContextType {
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
