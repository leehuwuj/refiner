'use client';

import React, { useEffect, useContext, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { CiSettings } from 'react-icons/ci';
import { Tooltip, Skeleton } from '@heroui/react';
import { LuClipboardCopy } from 'react-icons/lu';
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

async function tauri_get_correction_result(options: {
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

        const result = (await invoke('correct', payload)) as string;
        return result;
    } catch (error) {
        throw error;
    }
}

async function openMainWindow() {
    try {
        const mainWindow = await WebviewWindow.getByLabel('main');
        if (mainWindow) {
            await mainWindow.show();
            await mainWindow.setFocus();
        }
    } catch (error) {
        console.error('Error opening main window:', error);
    }
}

const TextSection = ({
    content,
    isTranslating,
}: {
    content: string;
    isTranslating?: boolean;
}) => {
    const [copied, setCopied] = React.useState(false);

    const handleCopy = (content: string) => {
        navigator.clipboard.writeText(content);
        setCopied(true);
        setTimeout(() => {
            setCopied(false);
        }, 2000);
    };

    return (
        <div className="relative h-full flex flex-col">
            {content && content.length > 0 && (
                <div className="absolute top-2 right-2 z-10">
                    <Tooltip isOpen={copied} content="Copied!">
                        <button onClick={() => handleCopy(content)}>
                            <LuClipboardCopy className="text-default-500 hover:text-default-700" size={16} />
                        </button>
                    </Tooltip>
                </div>
            )}

            <div className="overflow-auto scrollbar-hide p-2 h-full">
                {isTranslating && (
                    <div className="space-y-3">
                        <Skeleton className="w-3/5 rounded-lg">
                            <div className="h-3 w-3/5 rounded-lg bg-default-200"></div>
                        </Skeleton>
                        <Skeleton className="w-4/5 rounded-lg">
                            <div className="h-3 w-4/5 rounded-lg bg-default-200"></div>
                        </Skeleton>
                        <Skeleton className="w-2/5 rounded-lg">
                            <div className="h-3 w-2/5 rounded-lg bg-default-300"></div>
                        </Skeleton>
                    </div>
                )}

                <p className="text-xs text-default-600 leading-relaxed pr-6">
                    {content || (isTranslating ? '' : 'No content available')}
                </p>
            </div>
        </div>
    );
};

export default function CompactPopupClient() {
    const homeContext = useContext(TranslateContext);
    const appSettings = useContext(SettingContext);
    const isMounted = useRef(false);

    const homeContextRef = useRef(homeContext);
    const appSettingsRef = useRef(appSettings);

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
        let windowHandlerRef: ((event: Event) => void) | null = null;
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
                }
            });

            // Setup window event listener
            const handleShortcut = async (event: Event) => {
                if (!isHandlerActive) return;

                const rawText = (event as CustomEvent).detail as string;

                const currentHomeContext = homeContextRef.current;
                const currentAppSettings = appSettingsRef.current;

                currentHomeContext.changeResult({
                    translate: "...",
                    correct: "...",
                });

                try {
                    const translationResult = await tauri_get_result({
                        text: rawText,
                        provider: currentAppSettings.provider?.name,
                        model: currentAppSettings.model,
                        sourceLanguage: "<please detect source language (supported: English, Vietnamese)>",
                        targetLanguage: "<opposite with source language (supported: English, Vietnamese)>",
                    });

                    currentHomeContext.changeResult({
                        translate: translationResult,
                        correct: "...",
                    });

                    const correctionResult = await tauri_get_correction_result({
                        text: rawText,
                        provider: currentAppSettings.provider?.name,
                        model: currentAppSettings.model,
                        sourceLanguage: "<please detect source language (supported: English, Vietnamese)>",
                        targetLanguage: "<opposite with source language (supported: English, Vietnamese)>",
                    });

                    currentHomeContext.changeResult({
                        translate: translationResult,
                        correct: correctionResult,
                    });

                } catch (error) {
                    console.error('Translation error:', error);
                    currentHomeContext.changeResult({
                        translate: "Error occurred during translation",
                        correct: "Error occurred during correction",
                    });
                }
            };

            windowHandlerRef = handleShortcut;
            window.addEventListener("shortcut-popup-translate", handleShortcut as EventListener);
        };

        setupEventListeners();

        return () => {
            isHandlerActive = false;
            if (windowHandlerRef) {
                window.removeEventListener("shortcut-popup-translate", windowHandlerRef as EventListener);
            }
            if (tauriUnlisten) {
                tauriUnlisten();
            }
        };
    }, []);

    return (
        <div className="h-screen w-screen bg-transparent overflow-hidden">
            <div className="h-full w-full relative flex flex-col">
                <button
                    onClick={openMainWindow}
                    className="absolute bottom-2 right-2 z-10 p-1 rounded-full bg-default-100 hover:bg-default-200 transition-colors duration-200 shadow-sm"
                    title="Open Settings"
                >
                    <CiSettings size={16} className="text-default-600" />
                </button>

                <div className="flex-1 flex flex-col min-h-0">
                    <div className="flex-shrink-0 max-h-[40vh] min-h-[80px]">
                        <TextSection
                            content={homeContext.result?.translate || ""}
                            isTranslating={homeContext.translating && !homeContext.result?.translate}
                        />
                    </div>

                    <div className="w-full h-px bg-default-200 my-1" />

                    <div className="flex-1 min-h-0">
                        <TextSection
                            content={homeContext.result?.correct || ""}
                            isTranslating={homeContext.translating && !homeContext.result?.correct}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
} 