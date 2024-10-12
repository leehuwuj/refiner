"use client";

import { invoke } from "@tauri-apps/api/core";
import { Select, SelectItem } from "@nextui-org/react";
import { listen } from "@tauri-apps/api/event";
import { TextInput } from "./sections/input";
import { Result } from "./sections/result";
import React, { useCallback, useContext, useEffect, useState } from "react";
import { TranslateContext } from "./providers/translate";
import { BottomBar } from "./sections/bottomBar";
import { SettingContext } from "./providers/settings";
import { FaRegArrowAltCircleRight } from "react-icons/fa";
import { IoStopCircleOutline } from "react-icons/io5";
import { AppSettings } from "./types/settings";

async function tauri_get_result(
  text: string,
  provider: string,
  model: string,
  mode: Mode,
  sourceLanguage?: string,
  targetLanguage?: string,
): Promise<string> {
  const invoke_function = mode.toString().toLowerCase();
  console.log("Invoke function:", invoke_function);

  try {
    const payload = {
      provider: provider,
      model: model,
      text: text,
      sourceLang: sourceLanguage,
      targetLang: targetLanguage,
      prompt: null,
    };
    console.log("Payload:", payload);

    const result = (await invoke(invoke_function, payload)) as string;
    console.log("Result text:", result);
    return result;
  } catch (error) {
    console.error("Failed to invoke LLM:", error);
    throw error;
  }
}

const triggerTranslation = (
  homeContext: TranslateContextType,
  settingContext: AppSettings,
) => {
  homeContext.setTranslating(true);
  homeContext.changeResult({});
  // Get current text
  // const el = window.document.getElementById("input-text");
  tauri_get_result(
    homeContext.inputText ?? "",
    settingContext.provider?.name ?? "ollama",
    settingContext.model ?? "llama3",
    homeContext.currentMode,
    homeContext.languageConfig.sourceLang.label ?? "English",
    homeContext.languageConfig.targetLang.label ?? "Tiếng Việt",
  )
    .then((answer) => {
      // Change the mode result
      homeContext.changeResult({
        [homeContext.currentMode.toLocaleLowerCase()]: answer,
      });
    })
    .catch((error) => {
      console.error("Failed to translate text:", error);
    })
    .finally(() => {
      homeContext.setTranslating(false);
    });
};

async function startSerialEventListener() {
  await listen("shortcut-quickTranslate", (event) => {
    let rawText = event.payload as string;

    // Attempt to clean and extract the text
    if (typeof rawText === "string") {
      // Check if the text contains the expected prefix
      if (rawText.includes("text:")) {
        rawText = rawText.split("text:")[1].trim();
      }

      // Further clean the text if necessary
      rawText = rawText.replace(/^"|"$/g, '').trim(); // Remove surrounding quotes if present

      // Dispatch the cleaned text as an event
      window.dispatchEvent(
        new CustomEvent("shortcut-quickTranslate", { detail: rawText }),
      );
    } else {
      console.error("Invalid payload structure:", event.payload);
    }
  });
}

const LanguageSelections = ({
  selectedLang,
  changeLangConfig,
}: {
  selectedLang?: LanguageConfig;
  changeLangConfig: (lang: LanguageConfig) => void;
}) => {
  // const sourceLangues = [
  //   { value: "en", label: "English" },
  //   { value: "es", label: "Spanish" },
  //   { value: "fr", label: "French" },
  // ];
  const targetLangues = [
    { value: "vi", label: "Tiếng Việt" },
    { value: "en", label: "English" },
  ];

  return (
    <div className="mb-3 flex flex-row gap-2 align-middle items-center">
      <p className="w-3/6 text-gray-500 text-sm">
        Output language:
      </p>
      <Select
        className="w-3/6"
        value={selectedLang?.targetLang.label}
        placeholder={selectedLang?.targetLang.label}
      >
        {targetLangues.map((lang) => (
          <SelectItem
            key={lang.value}
            value={selectedLang?.targetLang.label}
            onClick={() => {
              const newConfig = selectedLang;
              if (newConfig) {
                newConfig.targetLang = {
                  code: lang.value,
                  label: lang.label,
                };
                changeLangConfig(newConfig);
              }
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
  const [triggerByShortcut, setTriggerByShortcut] = useState(false);

  const changeInputText = useCallback((text: string) => {
    homeContext.changeInputText(text);
  }, []);

  useEffect(() => {
    startSerialEventListener();
  }, []);

  useEffect(() => {
    if (triggerByShortcut) {
      setTriggerByShortcut(false);
      triggerTranslation(homeContext, appSettings);
    }
  }, [homeContext, appSettings, triggerByShortcut]);

  useEffect(() => {
    const handleShortcut = (event: any) => {
      let rawText = event.detail as string;
      console.log("Change input text: " + rawText);
      changeInputText(rawText);
      setTriggerByShortcut(true);
    };

    window.addEventListener("shortcut-quickTranslate", handleShortcut);
    return () => {
      window.removeEventListener("shortcut-quickTranslate", handleShortcut);
    };
  }, []);

  useEffect(() => {
    const handleKeyDown = async (event: any) => {
      if (event.key === "Escape") {
        console.log("Press escape key");

        await window.__TAURI__.window.getCurrent().hide();
      };
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

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
        triggerTranslation(homeContext, appSettings);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
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
            <TextInput
              inputText={homeContext.inputText}
              changeText={homeContext.changeInputText}
            />
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
            <button id="submit-trigger">
              <FaRegArrowAltCircleRight
                className="icon text-gray-500"
                size={24}
                onClick={() => triggerTranslation(homeContext, appSettings)}
              />
            </button>
          )}
        </div>
        <div className="mt-2 w-full sm:w-1/2">
          <Result
            result={homeContext.result}
            isTranslating={homeContext.translating}
            mode={homeContext.currentMode}
          />
        </div>
      </div>
      <BottomBar />
    </div>
  );
}
