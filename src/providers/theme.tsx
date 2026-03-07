import React, { createContext, useContext, useEffect, useState } from "react";
import type { ThemeType } from "@/types/settings";

// Namespaced key — ignores any stale "theme" values from old app runs
const THEME_KEY = "refiner-theme";

interface ThemeContextType {
  theme: ThemeType;
  resolvedTheme: "light" | "dark";
  setTheme: (theme: ThemeType) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used within ThemeProvider");
  return context;
};

function getInitialTheme(): ThemeType {
  const saved = localStorage.getItem(THEME_KEY) as ThemeType | null;
  return saved === "light" || saved === "dark" || saved === "system" ? saved : "dark";
}

function resolveTheme(t: ThemeType): "light" | "dark" {
  if (t === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return t;
}

function applyClass(resolved: "light" | "dark") {
  document.documentElement.classList.remove("light", "dark");
  document.documentElement.classList.add(resolved);
}

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [theme, setThemeState] = useState<ThemeType>(getInitialTheme);
  const [resolvedTheme, setResolvedTheme] = useState<"light" | "dark">(() =>
    resolveTheme(getInitialTheme()),
  );

  const setTheme = (newTheme: ThemeType) => {
    setThemeState(newTheme);
    const resolved = resolveTheme(newTheme);
    setResolvedTheme(resolved);
    localStorage.setItem(THEME_KEY, newTheme);
    applyClass(resolved);
  };

  useEffect(() => {
    applyClass(resolvedTheme);
  }, [resolvedTheme]);

  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      if (theme === "system") {
        const resolved = e.matches ? "dark" : "light";
        setResolvedTheme(resolved);
        applyClass(resolved);
      }
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};
