"use client";

import React from "react";
import { Button } from "@heroui/button";
import { Select, SelectItem } from "@heroui/select";
import { useTheme } from "../providers/theme";
import { ThemeType } from "../types/settings";

const themeOptions = [
    { key: "light", label: "Light" },
    { key: "dark", label: "Dark" },
    { key: "system", label: "System" },
] as const;

export const ThemeToggle: React.FC = () => {
    const { theme, setTheme } = useTheme();

    const handleThemeChange = (value: string) => {
        setTheme(value as ThemeType);
    };

    return (
        <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-foreground">Theme</label>
            <Select
                placeholder="Select theme"
                selectedKeys={[theme]}
                onSelectionChange={(keys) => {
                    const selectedKey = Array.from(keys)[0] as string;
                    if (selectedKey) {
                        handleThemeChange(selectedKey);
                    }
                }}
                className="max-w-xs"
            >
                {themeOptions.map((option) => (
                    <SelectItem key={option.key}>
                        {option.label}
                    </SelectItem>
                ))}
            </Select>
        </div>
    );
};

export default ThemeToggle; 