import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider } from "@/providers/theme";
import { SettingProvider } from "@/providers/settings";
import SettingsView from "@/views/SettingsView";
import "@/styles/globals.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider>
      <SettingProvider>
        <SettingsView />
      </SettingProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
