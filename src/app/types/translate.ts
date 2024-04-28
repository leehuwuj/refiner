interface Language {
  sourceLang?: string;
  targetLang?: string;
}

interface ResultTexts {
  translate?: string;
  correct?: string;
  refine?: string;
}

type Mode = "Translate" | "Correct" | "Refine";

interface TranslateContextType {
  languageConfig: Language;
  inputText?: string;
  result?: ResultTexts;
  translating?: boolean;
  currentMode: Mode;

  changeLangConfig: (lang: Language) => void;
  changeInputText: (text: string) => void;
  changeResult: (result: ResultTexts) => void;
  setTranslating: (isTranslating: boolean) => void;
  setCurrentMode: (mode: Mode) => void;
}
