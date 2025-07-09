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
import { useContext, useState } from "react";
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

  const handleSave = async (onClose: () => void) => {
    setIsSaving(true);
    setSaveMessage(null);

    try {
      // Get the API key
      const inputtedApiKey = document.getElementById("apiKey") as HTMLInputElement;
      const apiKey = inputtedApiKey.value.trim();

      // Use the context's save function
      const success = await settingContext.saveSettings(apiKey || undefined);

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
                    placeholder={settingContext.provider?.label ?? "Ollama"}
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
                    value={settingContext.model}
                    placeholder={settingContext.model}
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
                  type="secret"
                  label="API Key"
                  variant="bordered"
                />
                <div className="flex flex-col gap-2 justify-between">
                  <div className="flex flex-col">
                    <span className="text-sm font-medium">Shortcut Window Type</span>
                  </div>
                  <Select
                    value={settingContext.shortcutWindowType}
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
