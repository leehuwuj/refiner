"use client";

import { HeroUIProvider } from "@heroui/react";
import { TranslateProvider } from "./providers/translate";
import { SettingProvider } from "./providers/settings";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <HeroUIProvider>
      <SettingProvider>
        <TranslateProvider>{children}</TranslateProvider>
      </SettingProvider>
    </HeroUIProvider>
  );
}
