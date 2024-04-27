"use client";

import { invoke } from "@tauri-apps/api/tauri";
import { Button, Select, SelectItem } from "@nextui-org/react";
import { TextInput } from "./sections/input";
import { Result } from "./sections/result";
import React, { useContext, useEffect } from "react";
import { TranslateContext } from "./providers/translate";
import { BottomBar } from "./sections/bottomBar";
import { SettingContext } from "./providers/settings";
import { FaRegArrowAltCircleRight, FaTimes } from "react-icons/fa";
import { IoStopCircleOutline } from "react-icons/io5";

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
      <Select
        value={selectedLang?.sourceLang}
        placeholder={selectedLang?.sourceLang}
      >
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
      <Select
        value={selectedLang?.targetLang}
        placeholder={selectedLang?.targetLang}
      >
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

  const triggerTranslation = () => {
    homeContext.setTranslating(true);
    // Clear result
    homeContext.changeResult({});
    tauri_translateText(
      homeContext.inputText ?? "",
      appSettings.provider?.name ?? "ollama",
      appSettings.model ?? "llama3",
      homeContext.languageConfig.targetLang ?? "vi",
    )
      .then((translated) => {
        homeContext.changeResult({ translated: translated });
      })
      .catch((error) => {
        console.error("Failed to translate text:", error);
      })
      .finally(() => {
        homeContext.setTranslating(false);
      });
  };

  useEffect(() => {
    const handleKeyDown = (event: {
      ctrlKey: any;
      metaKey: any;
      key: string;
    }) => {
      if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "Enter" &&
        !homeContext.translating
      ) {
        triggerTranslation();
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    // Cleanup on unmount
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [homeContext, appSettings]);

  return (
    <div className="flex flex-col">
      <div className="flex h-80 w-full flex-col p-2 sm:flex-row">
        <div className="m-2 flex h-full w-full flex-col sm:w-1/2">
          <LanguageSelections
            selectedLang={homeContext.languageConfig}
            changeLangConfig={homeContext.changeLangConfig}
          />
          <div className="mb-0 h-80 overflow-clip">
            <TextInput changeText={homeContext.changeInputText} />
          </div>
        </div>
        <div className="flex w-8 flex-col items-center justify-center space-y-5 pt-10">
          {homeContext.translating ? (
            <IoStopCircleOutline
              className="icon text-gray-500"
              size={24}
              onClick={() => {
                homeContext.setTranslating(false);
              }}
            />
          ) : (
            <FaRegArrowAltCircleRight
              className="icon text-gray-500"
              size={24}
              onClick={() => triggerTranslation()}
            />
          )}
        </div>
        <div className="mt-2 w-full sm:w-1/2">
          <Result
            result={homeContext.result}
            isTranslating={homeContext.translating}
          />
        </div>
      </div>
      <BottomBar />
    </div>
  );
}
