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

const Settings = ({
  isOpen,
  onOpen,
  onOpenChange,
}: {
  isOpen: boolean;
  onOpen: () => void;
  onOpenChange: () => void;
}) => {
  return (
    <>
      <Modal
        isOpen={isOpen}
        onOpenChange={onOpenChange}
        placement="top-center"
        size="lg"
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
                    placeholder="Select a LLM provider "
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
                    placeholder="Select a model"
                    variant="bordered"
                    className="w-1/2"
                    aria-label="Select a model"
                  >
                    <SelectItem value="Phi 3" key="phi3" id="phi3">
                      Phi 3
                    </SelectItem>
                    <SelectItem value="Ollama 3" key="ollama3" id="phi3">
                      Ollama 3
                    </SelectItem>
                  </Select>
                </div>
                <div className="justify-between px-1 py-2">
                  <Checkbox classNames={{ label: "text-small" }}>
                    Use my custom prompt
                  </Checkbox>
                  <Textarea
                    variant="bordered"
                    placeholder="Input your custom prompt here"
                    disableAutosize
                    classNames={{
                      input: "h-40", // Increase the height here
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
