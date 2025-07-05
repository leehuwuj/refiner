import React from "react";
import { Select, SelectItem } from "@heroui/react";
import { HiSwitchHorizontal } from "react-icons/hi";

interface LanguageConfig {
    sourceLang: { code: string; label: string };
    targetLang: { code: string; label: string };
}

const LanguageSelections = ({
    selectedLang,
    changeLangConfig,
}: {
    selectedLang?: LanguageConfig;
    changeLangConfig: (lang: LanguageConfig) => void;
}) => {
    const targetLanguages = [
        { value: "vi", label: "Tiếng Việt" },
        { value: "en", label: "English" },
    ];

    const handleLanguageSwitch = () => {
        if (selectedLang) {
            const newConfig = {
                sourceLang: selectedLang.targetLang,
                targetLang: selectedLang.sourceLang,
            };
            changeLangConfig(newConfig);
        }
    };

    return (
        <div className="mb-3 flex flex-row gap-2 align-middle items-center">
            <div className="flex w-3/6 items-center relative space-x-2">
                <Select
                    className="flex-grow"
                    placeholder={selectedLang?.targetLang.label}
                >
                    {targetLanguages.map((lang) => (
                        <SelectItem
                            key={lang.value}
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
                <button
                    onClick={handleLanguageSwitch}
                    aria-label="Switch Languages"
                >
                    <HiSwitchHorizontal size={20} />
                </button>
            </div>
        </div>
    );
};

export { LanguageSelections };
