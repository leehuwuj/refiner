'use client';

import { useEffect, useContext, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Result } from '../sections/result';
import { TranslateContext } from '../providers/translate';
import { SettingContext } from '../providers/settings';

async function tauri_get_result(
    text: string,
    provider: string,
    model: string,
    sourceLanguage?: string,
    targetLanguage?: string,
): Promise<string> {
    try {
        const payload = {
            provider: provider,
            model: model,
            text: text,
            sourceLang: sourceLanguage,
            targetLang: targetLanguage,
            prompt: null,
        };
        const result = (await invoke('translate', payload)) as string;
        return result;
    } catch (error) {
        console.error("Failed to invoke LLM:", error);
        throw error;
    }
}

export default function TranslatePopupClient() {
    const homeContext = useContext(TranslateContext);
    const settingsContext = useContext(SettingContext);

    const changeInputText = useCallback((text: string) => {
        homeContext.changeInputText(text);
    }, []);

    useEffect(() => {
        const unlisten = listen('shortcut-quickTranslate', (event) => {
            const payload = event.payload as { text: string };
            if (!payload || typeof payload.text !== 'string') {
                console.error('Invalid payload received:', event.payload);
                return;
            }

            const text = payload.text.trim();
            changeInputText(text);
            homeContext.setCurrentMode('Translate');

            // Trigger translation
            homeContext.setTranslating(true);
            homeContext.changeResult({});

            tauri_get_result(
                text,
                settingsContext.provider?.name ?? "ollama",
                settingsContext.model ?? "llama3",
                homeContext.languageConfig.sourceLang.label ?? "English",
                homeContext.languageConfig.targetLang.label ?? "Tiếng Việt",
            )
                .then((answer) => {
                    homeContext.changeResult({
                        translate: answer,
                    });
                })
                .catch((error) => {
                    console.error("Failed to translate text:", error);
                })
                .finally(() => {
                    homeContext.setTranslating(false);
                });
        });

        return () => {
            unlisten.then(fn => fn());
        };
    }, [homeContext, settingsContext]);

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

    return (
        <div className="h-screen w-screen bg-transparent">
            <div className="h-full w-full">
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