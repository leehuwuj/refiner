"use client";

import { HeroUIProvider } from "@heroui/react";
import { TranslateProvider } from "./providers/translate";
import { SettingProvider } from "./providers/settings";
import { ThemeProvider } from "./providers/theme";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider>
      <HeroUIProvider>
        <SettingProvider>
          <TranslateProvider>{children}</TranslateProvider>
        </SettingProvider>
      </HeroUIProvider>
    </ThemeProvider>
  );
}
