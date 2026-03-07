import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider } from "@/providers/theme";
import { SettingProvider } from "@/providers/settings";
import { TranslateProvider } from "@/providers/translate";
import MainView from "@/views/MainView";
import "@/styles/globals.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider>
      <SettingProvider>
        <TranslateProvider>
          <MainView />
        </TranslateProvider>
      </SettingProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
