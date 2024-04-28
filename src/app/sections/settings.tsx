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
import { invoke } from "@tauri-apps/api/core";
import { useContext } from "react";
import { SettingContext } from "../providers/settings";
import { providerMap } from "../types/settings";

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
                <Input id="apiKey" type="secret" label="API Key" variant="bordered" />
                {/* <div className="justify-between px-1 py-2">
                  <Checkbox disabled classNames={{ label: "text-small" }}>
                    Use my custom prompt
                  </Checkbox>
                  <Textarea
                    disabled
                    variant="bordered"
                    placeholder="This feature is in development."
                    disableAutosize
                    classNames={{
                      input: "h-20", // Increase the height here
                    }}
                  />
                </div> */}
              </ModalBody>
              <ModalFooter>
                {/* call api to save api key */}
                <Button color="primary" onClick={() => {
                  // Get the API key
                  const apiKey = document.getElementById("apiKey") as HTMLInputElement;
                  console.log(apiKey.value);
                  // Save the API key
                  invoke("save_api_key", { apiKey: apiKey.value });
                  onClose();
                }}>
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
