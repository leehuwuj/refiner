'use client';

import { useEffect, useContext } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { CiSettings } from 'react-icons/ci';
import { Result } from '../sections/result';
import { TranslateContext } from '../providers/translate';
import { SettingContext } from '../providers/settings';

async function tauri_get_result(options: {
    text: string,
    provider: string,
    model: string,
    sourceLanguage?: string,
    targetLanguage?: string,
}): Promise<string> {
    try {
        const payload = {
            provider: options.provider,
            model: options.model,
            text: options.text,
            sourceLang: options.sourceLanguage,
            targetLang: options.targetLanguage,
            prompt: null,
        };
        const result = (await invoke('translate', payload)) as string;
        return result;
    } catch (error) {
        console.error("Failed to invoke LLM:", error);
        throw error;
    }
}

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

async function openMainWindow() {
    try {
        const mainWindow = await WebviewWindow.getByLabel('main');
        if (mainWindow) {
            await mainWindow.show();
            await mainWindow.setFocus();
        } else {
            console.error('Main window not found');
        }
    } catch (error) {
        console.error('Failed to open main window:', error);
    }
}

export default function TranslatePopupClient() {
    const homeContext = useContext(TranslateContext);
    const appSettings = useContext(SettingContext);

    useEffect(() => {
        startSerialEventListener();
    }, []);

    useEffect(() => {
        const handleKeyDown = async (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                try {
                    const window = await WebviewWindow.getByLabel('translate-popup');
                    if (window) {
                        await window.hide();
                    }
                } catch (error) {
                    console.error('Failed to hide window:', error);
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => {
            window.removeEventListener('keydown', handleKeyDown);
        };
    }, []);

    useEffect(() => {
        const handleShortcut = async (event: any) => {
            let rawText = event.detail as string;
            homeContext.changeResult({
                [homeContext.currentMode.toLocaleLowerCase()]: "...",
            });
            // Trigger translation
            const res = await tauri_get_result(
                {
                    text: rawText,
                    provider: appSettings.provider?.name ?? "ollama",
                    model: appSettings.model ?? "llama3",
                    sourceLanguage: homeContext.languageConfig.sourceLang.code ?? "en",
                    targetLanguage: homeContext.languageConfig.targetLang.code ?? "vi",
                }
            );
            homeContext.changeResult({
                [homeContext.currentMode.toLocaleLowerCase()]: res,
            });
        };
        window.addEventListener("shortcut-quickTranslate", handleShortcut);
        return () => {
            window.removeEventListener("shortcut-quickTranslate", handleShortcut);
        };
    }, []);

    return (
        <div className="h-screen w-screen bg-transparent">
            <div className="h-full w-full relative">
                {/* Settings button */}
                <button
                    onClick={openMainWindow}
                    className="absolute bottom-2 right-2 z-10 p-1 rounded-full bg-gray-200 hover:bg-gray-300 transition-colors duration-200 shadow-sm"
                    title="Open Settings"
                >
                    <CiSettings size={16} className="text-gray-600" />
                </button>

                <Result
                    result={homeContext.result}
                    isTranslating={homeContext.translating}
                    mode={homeContext.currentMode}
                    compact={true}
                />
            </div>
        </div>
    );
} 