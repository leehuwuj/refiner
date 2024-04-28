interface Language {
  sourceLang?: string;
  targetLang?: string;
}

interface ResultTexts {
  translated?: string;
  correct?: string;
  refine?: string;
}

interface TranslateContextType {
  languageConfig: Language;
  inputText?: string;
  result?: ResultTexts;
  translating?: boolean;

  changeLangConfig: (lang: Language) => void;
  changeInputText: (text: string) => void;
  changeResult: (result: ResultTexts) => void;
  setTranslating: (isTranslating: boolean) => void;
}
