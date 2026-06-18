import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider } from "@/providers/theme";
import { SettingProvider } from "@/providers/settings";
import LanguageAnalysisView from "@/views/LanguageAnalysisView";
import "@/styles/globals.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider>
      <SettingProvider>
        <LanguageAnalysisView />
      </SettingProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
