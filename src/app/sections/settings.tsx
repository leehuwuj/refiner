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
} from "@nextui-org/react";
import { Store } from "@tauri-apps/plugin-store";
import { useContext } from "react";
import { SettingContext } from "../providers/settings";
import { providerMap } from "../types/settings";

const store_api_key = async (api_key: string) => {
  const store = new Store("store.bin");
  store.set("OPENAI_API_KEY", api_key);
  await store.save();
  console.log("Stored API key");
  let duma = await store.get("OPENAI_API_KEY");
  console.log("Stored key: ", duma);
};

const Settings = ({
  isOpen,
  onOpen,
  onOpenChange,
}: {
  isOpen: boolean;
  onOpen: () => void;
  onOpenChange: () => void;
}) => {
  const settingContext = useContext(SettingContext);

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
                    <SelectItem value="Ollama" key="ollama">
                      Ollama
                    </SelectItem>
                    <SelectItem value="OpenAI" key="openai">
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
                      <SelectItem value={model} key={model} id={model}>
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
              </ModalBody>
              <ModalFooter>
                {/* call api to save api key */}
                <Button
                  color="primary"
                  onClick={async () => {
                    // Get the API key
                    const inputtedApiKey = document.getElementById("apiKey") as HTMLInputElement;
                    const apiKey = inputtedApiKey.value.trim();
                    // Store the API key
                    await store_api_key(
                      apiKey,
                    );
                    onClose();
                  }}
                >
                  Save
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
