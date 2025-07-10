'use client';

import { useEffect, useContext, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { CiSettings } from 'react-icons/ci';
import { Result } from '../sections/result';
import { TranslateContext } from '../providers/translate';
import { SettingContext } from '../providers/settings';

async function tauri_get_result(options: {
    text: string,
    provider?: string,
    model?: string,
    sourceLanguage?: string,
    targetLanguage?: string,
}): Promise<string> {
    try {
        const payload = {
            provider: options.provider || null,
            model: options.model || null,
            text: options.text,
            sourceLang: options.sourceLanguage,
            targetLang: options.targetLanguage,
            prompt: null,
        };

        const result = (await invoke('translate', payload)) as string;
        return result;
    } catch (error) {
        throw error;
    }
}

async function startSerialEventListener() {
    await listen("shortcut-popup-translate", (event) => {
        let rawText = event.payload as string;
        if (typeof rawText === "string") {
            if (rawText.includes("text:")) {
                rawText = rawText.split("text:")[1].trim();
            }
            rawText = rawText.replace(/^"|"$/g, '').trim();

            window.dispatchEvent(
                new CustomEvent("shortcut-popup-translate", { detail: rawText }),
            );
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
        }
    } catch (error) {
    }
}

export default function TranslatePopupClient() {
    const homeContext = useContext(TranslateContext);
    const appSettings = useContext(SettingContext);
    const isMounted = useRef(false);

    // Refs to store latest values
    const homeContextRef = useRef(homeContext);
    const appSettingsRef = useRef(appSettings);

    // Update refs when values change
    useEffect(() => {
        homeContextRef.current = homeContext;
    }, [homeContext]);

    useEffect(() => {
        appSettingsRef.current = appSettings;
    }, [appSettings]);

    useEffect(() => {
        if (isMounted.current) return;
        isMounted.current = true;

        let tauriUnlisten: (() => void) | null = null;
        let windowHandlerRef: ((event: any) => void) | null = null;
        let isHandlerActive = true;

        const setupEventListeners = async () => {
            // Setup Tauri event listener
            tauriUnlisten = await listen("shortcut-popup-translate", (event) => {
                if (!isHandlerActive) return;

                let rawText = event.payload as string;
                if (typeof rawText === "string") {
                    if (rawText.includes("text:")) {
                        rawText = rawText.split("text:")[1].trim();
                    }
                    rawText = rawText.replace(/^"|"$/g, '').trim();

                    window.dispatchEvent(
                        new CustomEvent("shortcut-popup-translate", { detail: rawText }),
                    );
                } else {
                }
            });

            // Setup window event listener
            const handleShortcut = async (event: any) => {
                if (!isHandlerActive) return;

                let rawText = event.detail as string;

                // Use current values from refs
                const currentHomeContext = homeContextRef.current;
                const currentAppSettings = appSettingsRef.current;

                currentHomeContext.changeResult({
                    [currentHomeContext.currentMode.toLocaleLowerCase()]: "...",
                });

                const res = await tauri_get_result({
                    text: rawText,
                    provider: currentAppSettings.provider?.name,
                    model: currentAppSettings.model,
                    sourceLanguage: currentHomeContext.languageConfig.sourceLang.code ?? "en",
                    targetLanguage: currentHomeContext.languageConfig.targetLang.code ?? "vi",
                });

                currentHomeContext.changeResult({
                    [currentHomeContext.currentMode.toLocaleLowerCase()]: res,
                });
            };

            windowHandlerRef = handleShortcut;
            window.addEventListener("shortcut-popup-translate", handleShortcut);
        };

        setupEventListeners();

        return () => {
            isHandlerActive = false;
            if (windowHandlerRef) {
                window.removeEventListener("shortcut-popup-translate", windowHandlerRef);
            }
            if (tauriUnlisten) {
                tauriUnlisten();
            }
        };
    }, []);

    return (
        <div className="h-screen w-screen bg-transparent overflow-auto scrollbar-hide">
            <div className="h-full w-full relative overflow-auto scrollbar-hide">
                {/* Settings button */}
                <button
                    onClick={openMainWindow}
                    className="absolute bottom-2 right-2 z-10 p-1 rounded-full bg-default-100 hover:bg-default-200 transition-colors duration-200 shadow-sm"
                    title="Open Settings"
                >
                    <CiSettings size={16} className="text-default-600" />
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