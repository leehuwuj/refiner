import {
  Modal,
  ModalHeader,
  ModalContent,
  ModalFooter,
  Button,
  ModalBody,
  Checkbox,
  Select,
  SelectItem,
  Textarea,
} from "@nextui-org/react";
import { useContext } from "react";
import { SettingContext } from "../providers/settings";

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
                    placeholder={settingContext.provider?.name ?? "Ollama"}
                    variant="bordered"
                    className="w-1/2"
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
                  >
                    {settingContext.provider?.models?.map((model) => (
                      <SelectItem value={model} key={model} id={model}>
                        {model}
                      </SelectItem>
                    )) || []}
                  </Select>
                </div>
                <div className="justify-between px-1 py-2">
                  <Checkbox classNames={{ label: "text-small" }}>
                    Use my custom prompt
                  </Checkbox>
                  <Textarea
                    variant="bordered"
                    placeholder="Ex: You're a nice translator!"
                    disableAutosize
                    classNames={{
                      input: "h-20", // Increase the height here
                    }}
                  />
                </div>
              </ModalBody>
              <ModalFooter>
                <Button color="primary" onPress={onClose}>
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
