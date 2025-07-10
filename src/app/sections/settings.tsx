import {
  Modal,
  ModalHeader,
  ModalContent,
  ModalFooter,
  Button,
  ModalBody,
  Select,
  SelectItem,
  Input,
} from "@heroui/react";
import { useContext, useState, useEffect } from "react";
import { SettingContext } from "../providers/settings";
import { providerMap, ShortcutWindowType } from "../types/settings";
import ThemeToggle from "../components/theme-toggle";

const Settings = ({
  isOpen,
  onOpenChange,
}: {
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
}) => {
  const settingContext = useContext(SettingContext);
  const [isSaving, setIsSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [localApiKey, setLocalApiKey] = useState<string>("");

  // Update local API key when context changes
  useEffect(() => {
    if (settingContext.apiKey) {
      setLocalApiKey(settingContext.apiKey);
    }
  }, [settingContext.apiKey]);

  const handleSave = async (onClose: () => void) => {
    setIsSaving(true);
    setSaveMessage(null);

    try {
      // Use the context's save function
      const success = await settingContext.saveSettings(localApiKey || undefined);

      if (success) {
        setSaveMessage("Settings saved successfully!");

        // Close modal after a short delay to show the success message
        setTimeout(() => {
          onClose();
          setSaveMessage(null);
        }, 1500);
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
    <>
      <Modal
        isOpen={isOpen}
        onOpenChange={onOpenChange}
        placement="top-center"
        size="sm"
      >
        <ModalContent>
          {(onClose) => (
            <>
              <ModalHeader className="flex flex-col gap-1">
                Settings:{" "}
              </ModalHeader>
              <ModalBody>
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
                      // Change provider also changes the model
                      if (provider.models) {
                        settingContext.setModel(provider.models[0]);
                      }
                    }}
                  >
                    <SelectItem key="ollama">
                      Ollama
                    </SelectItem>
                    <SelectItem key="openai">
                      OpenAI
                    </SelectItem>
                  </Select>
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
                </div>
                <Input
                  id="apiKey"
                  type="password"
                  label="API Key"
                  variant="bordered"
                  value={localApiKey}
                  onChange={(e) => setLocalApiKey(e.target.value)}
                />
                <div className="flex flex-col gap-2 justify-between">
                  <div className="flex flex-col">
                    <span className="text-sm font-medium">Shortcut Window Type</span>
                  </div>
                  <Select
                    selectedKeys={settingContext.shortcutWindowType ? [settingContext.shortcutWindowType] : []}
                    className="w-full"
                    placeholder="Select"
                    size="sm"
                    onChange={(e) => {
                      settingContext.setShortcutWindowType(e.target.value as ShortcutWindowType);
                    }}
                  >
                    <SelectItem key="popup">
                      Small Popup
                    </SelectItem>
                    <SelectItem key="main">
                      Main Window
                    </SelectItem>
                  </Select>
                </div>
                <ThemeToggle />
                {saveMessage && (
                  <div className={`text-sm p-2 rounded ${saveMessage.includes("Failed")
                    ? "bg-red-100 text-red-700 border border-red-300"
                    : "bg-green-100 text-green-700 border border-green-300"
                    }`}>
                    {saveMessage}
                  </div>
                )}
              </ModalBody>
              <ModalFooter>
                <Button
                  color="primary"
                  onPress={() => handleSave(onClose)}
                  isLoading={isSaving}
                  disabled={isSaving}
                >
                  {isSaving ? "Saving..." : "Save"}
                </Button>
              </ModalFooter>
            </>
          )}
        </ModalContent>
      </Modal>
    </>
  );
};

export { Settings };
