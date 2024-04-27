"use client";

import { invoke } from "@tauri-apps/api/tauri";
import { Button, Select, SelectItem } from "@nextui-org/react";
import { TextInput } from "./sections/input";
import { Result } from "./sections/result";
import React, { useContext } from "react";
import { TranslateContext } from "./providers/translate";
import { BottomBar } from "./sections/bottomBar";
import { SettingContext } from "./providers/settings";

async function tauri_translateText(
  text: string,
  provider: string,
  model: string,
  targetLanguage: string,
): Promise<string> {
  try {
    const result = (await invoke("translate", {
      provider: provider,
      model: model,
      text: text,
    })) as string;
    console.log("Translated text:", result);
    return result;
  } catch (error) {
    console.error("Failed to translate text:", error);
    throw error;
  }
}

const LanguageSelections = ({
  selectedLang,
  changeLangConfig,
}: {
  selectedLang?: Language;
  changeLangConfig: (lang: Language) => void;
}) => {
  const sourceLangues = [
    { value: "en", label: "English" },
    { value: "es", label: "Spanish" },
    { value: "fr", label: "French" },
  ];
  const targetLangues = [
    { value: "vi", label: "Vietnamese" },
    { value: "en", label: "English" },
  ];

  return (
    <div className="mb-3 flex flex-wrap gap-2 md:flex-nowrap">
      <Select value={selectedLang?.sourceLang} placeholder={selectedLang?.sourceLang} >
        {sourceLangues.map((lang) => (
          <SelectItem
            key={lang.value}
            value={selectedLang?.sourceLang}
            onClick={() => {
              changeLangConfig({ ...selectedLang, sourceLang: lang.value });
            }}
            id={lang.label}
          >
            {lang.label}
          </SelectItem>
        ))}
      </Select>
      <p className="self-center text-center text-gray-500">{" => "}</p>
      <Select value={selectedLang?.targetLang} placeholder={selectedLang?.targetLang}>
        {targetLangues.map((lang) => (
          <SelectItem
            key={lang.value}
            value={selectedLang?.targetLang}
            onClick={() => {
              changeLangConfig({ ...selectedLang, targetLang: lang.value });
            }}
            id={lang.label}
          >
            {lang.label}
          </SelectItem>
        ))}
      </Select>
    </div>
  );
};

export default function Home() {
  const homeContext = useContext(TranslateContext);
  const appSettings = useContext(SettingContext);

  return (
    <div>
      <div className="flex h-60 max-h-60 flex-col p-2 sm:flex-row">
        <div className="m-2 w-full sm:w-1/2">
          <LanguageSelections
            selectedLang={homeContext.languageConfig}
            changeLangConfig={homeContext.changeLangConfig}
          />
          <TextInput changeText={homeContext.changeInputText} />
        </div>
        <div className="m-2 w-full sm:w-1/2">
          <Result result={homeContext.result} />
        </div>
      </div>
      <div className="ml-4 mt-10">
        <Button
          onClick={() => {
            tauri_translateText(
              homeContext.inputText ?? "",
              appSettings.provider?.name ?? "ollama",
              appSettings.model ?? "phi3",
              homeContext.languageConfig.targetLang ?? "vi",
            )
              .then((translated) => {
                homeContext.changeResult({ translated: translated });
              })
              .catch((error) => {
                console.error("Failed to translate text:", error);
              });
          }}
        >
          Translate
        </Button>
      </div>
      <BottomBar />
    </div>
  );
}
