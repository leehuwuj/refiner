"use client";

import { NextUIProvider } from "@nextui-org/react";
import { TranslateProvider } from "./providers/translate";
import { SettingProvider } from "./providers/settings";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <NextUIProvider>
      <SettingProvider>
        <TranslateProvider>{children}</TranslateProvider>
      </SettingProvider>
    </NextUIProvider>
  );
}
