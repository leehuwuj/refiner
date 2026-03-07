"use client";

import {
  Button,
  Select,
  SelectItem,
  Input,
  Switch,
} from "@heroui/react";
import { useContext, useState, useEffect } from "react";
import { SettingContext } from "../providers/settings";
import { providerMap, ShortcutWindowType } from "../types/settings";
import ThemeToggle from "../components/theme-toggle";

const SettingsClient = () => {
  const settingContext = useContext(SettingContext);
  const [isSaving, setIsSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [localApiKey, setLocalApiKey] = useState<string>("");

  const isOllama = settingContext.provider?.name === "ollama";

  useEffect(() => {
    if (settingContext.apiKey) {
      setLocalApiKey(settingContext.apiKey);
    }
  }, [settingContext.apiKey]);

  const handleSave = async () => {
    setIsSaving(true);
    setSaveMessage(null);

    try {
      const success = await settingContext.saveSettings(localApiKey || undefined);

      if (success) {
        setSaveMessage("Settings saved successfully!");
        setTimeout(() => setSaveMessage(null), 2000);
      } else {
        setSaveMessage("Failed to save settings. Please try again.");
      }
    } catch (error) {
      console.error("Failed to save settings:", error);
      setSaveMessage("Failed to save settings. Please try again.");
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      <h1 className="text-lg font-semibold">Settings</h1>

      <div className="flex w-full space-x-2">
        <Select
          autoFocus
          label="Provider"
          selectedKeys={settingContext.provider ? [settingContext.provider.name] : []}
          placeholder={settingContext.provider?.label ?? "Select Provider"}
          variant="bordered"
          className="w-1/2"
          onChange={(e) => {
            const selectedProviderName = e.target.value.toLowerCase();
            const provider = providerMap[selectedProviderName];
            settingContext.setProvider(provider);
            if (provider.models) {
              settingContext.setModel(provider.models[0]);
            }
          }}
        >
          <SelectItem key="ollama">Ollama</SelectItem>
          <SelectItem key="openai">OpenAI</SelectItem>
          <SelectItem key="gemini">Gemini</SelectItem>
          <SelectItem key="groq">Groq</SelectItem>
        </Select>
        {isOllama ? (
          <Input
            label="Model"
            variant="bordered"
            className="w-1/2"
            placeholder="e.g. gemma3, llama3:8b"
            value={settingContext.model ?? ""}
            onChange={(e) => settingContext.setModel(e.target.value)}
            description={
              <span className="text-xs text-default-400">
                Suggestions: {providerMap["ollama"].models?.join(", ")}
              </span>
            }
          />
        ) : (
          <Select
            label="Model"
            variant="bordered"
            className="w-1/2"
            aria-label="Select a model"
            selectedKeys={settingContext.model ? [settingContext.model] : []}
            placeholder={settingContext.model || "Select Model"}
            onChange={(e) => {
              settingContext.setModel(e.target.value);
            }}
          >
            {settingContext.provider?.models?.map((model) => (
              <SelectItem key={model} id={model}>
                {model}
              </SelectItem>
            )) || []}
          </Select>
        )}
      </div>

      {isOllama && (
        <>
          <Input
            label="Ollama Endpoint"
            variant="bordered"
            value={settingContext.ollamaEndpoint ?? "http://localhost:11434"}
            onChange={(e) => settingContext.setOllamaEndpoint(e.target.value)}
            placeholder="http://localhost:11434"
          />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Enable Thinking</p>
              <p className="text-xs text-default-400">For models that support extended reasoning (e.g. qwen3)</p>
            </div>
            <Switch
              isSelected={settingContext.ollamaThinking ?? true}
              onValueChange={(v) => settingContext.setOllamaThinking(v)}
            />
          </div>
        </>
      )}

      {!isOllama && (
        <Input
          id="apiKey"
          type="password"
          label="API Key"
          variant="bordered"
          value={localApiKey}
          onChange={(e) => setLocalApiKey(e.target.value)}
        />
      )}

      <div className="flex flex-col gap-2">
        <span className="text-sm font-medium">Shortcut Window Type</span>
        <Select
          selectedKeys={settingContext.shortcutWindowType ? [settingContext.shortcutWindowType] : []}
          className="w-full"
          placeholder="Select"
          size="sm"
          onChange={(e) => {
            settingContext.setShortcutWindowType(e.target.value as ShortcutWindowType);
          }}
        >
          <SelectItem key="popup">Small Popup</SelectItem>
          <SelectItem key="main">Main Window</SelectItem>
        </Select>
      </div>

      <ThemeToggle />

      {saveMessage && (
        <div
          className={`text-sm p-2 rounded ${
            saveMessage.includes("Failed")
              ? "bg-red-100 text-red-700 border border-red-300"
              : "bg-green-100 text-green-700 border border-green-300"
          }`}
        >
          {saveMessage}
        </div>
      )}

      <div className="mt-auto">
        <Button
          color="primary"
          onPress={handleSave}
          isLoading={isSaving}
          disabled={isSaving}
          className="w-full"
        >
          {isSaving ? "Saving..." : "Save"}
        </Button>
      </div>
    </div>
  );
};

export default SettingsClient;
