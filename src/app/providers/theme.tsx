"use client";

import React, { createContext, useContext, useEffect, useState } from "react";
import { ThemeType } from "../types/settings";

interface ThemeContextType {
    theme: ThemeType;
    resolvedTheme: "light" | "dark";
    setTheme: (theme: ThemeType) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
    const context = useContext(ThemeContext);
    if (!context) {
        throw new Error("useTheme must be used within a ThemeProvider");
    }
    return context;
};

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [theme, setTheme] = useState<ThemeType>("system");
    const [resolvedTheme, setResolvedTheme] = useState<"light" | "dark">("light");
    const [mounted, setMounted] = useState(false);

    // System theme detection
    const getSystemTheme = (): "light" | "dark" => {
        if (typeof window !== "undefined") {
            return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
        }
        return "light";
    };

    // Update resolved theme based on current theme setting
    const updateResolvedTheme = (currentTheme: ThemeType) => {
        if (currentTheme === "system") {
            setResolvedTheme(getSystemTheme());
        } else {
            setResolvedTheme(currentTheme);
        }
    };

    // Handle theme changes
    const handleThemeChange = (newTheme: ThemeType) => {
        setTheme(newTheme);
        updateResolvedTheme(newTheme);

        // Save to localStorage
        if (typeof window !== "undefined") {
            localStorage.setItem("theme", newTheme);
        }
    };

    // Initialize theme from localStorage or default to system
    useEffect(() => {
        const savedTheme = (typeof window !== "undefined" ? localStorage.getItem("theme") : null) as ThemeType | null;
        const initialTheme = savedTheme || "system";

        setTheme(initialTheme);
        updateResolvedTheme(initialTheme);
        setMounted(true);
    }, []);

    // Listen for system theme changes
    useEffect(() => {
        if (typeof window === "undefined") return;

        const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

        const handleSystemThemeChange = (e: MediaQueryListEvent) => {
            if (theme === "system") {
                setResolvedTheme(e.matches ? "dark" : "light");
            }
        };

        mediaQuery.addEventListener("change", handleSystemThemeChange);

        return () => {
            mediaQuery.removeEventListener("change", handleSystemThemeChange);
        };
    }, [theme]);

    // Apply theme class to document
    useEffect(() => {
        if (!mounted) return;

        const root = document.documentElement;

        // Remove existing theme classes
        root.classList.remove("light", "dark");

        // Add current theme class
        root.classList.add(resolvedTheme);
    }, [resolvedTheme, mounted]);

    // Don't render theme-sensitive content until mounted to avoid hydration mismatch
    if (!mounted) {
        return <div className="opacity-0">{children}</div>;
    }

    return (
        <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme: handleThemeChange }}>
            {children}
        </ThemeContext.Provider>
    );
}; 