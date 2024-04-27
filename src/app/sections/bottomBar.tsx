import { Select, SelectItem } from "@nextui-org/select";
import { CiSettings } from "react-icons/ci";
import { Settings } from "./settings";
import React from "react";

const BottomBar = () => {
  const [openSettings, setOpenSettings] = React.useState(false);

  return (
    <div>
      <div className="fixed bottom-0 left-0 flex h-12 w-full items-center justify-between bg-gray-200 pl-3 pr-3">
        <div className="flex items-center space-x-3">
          <button
            onClick={() => setOpenSettings(true)}
            className="text-gray-500"
          >
            <CiSettings size={24} className="font-bold" />
          </button>
          <Select
            size="sm"
            placeholder="Phi 3"
            value={"Phi 3"}
            className="w-40"
          >
            <SelectItem value="Phi 3" key={"phi3"}>
              Phi 3
            </SelectItem>
            <SelectItem value="Ollama 3" key={"ollama3"}>
              Ollama 3
            </SelectItem>
          </Select>
        </div>
        <p className="flex items-center text-sm text-gray-500">
          © 2024 Refine App, Plaida.com LLC.
        </p>
      </div>
      {openSettings && (
        <Settings
          isOpen={openSettings}
          onOpen={() => setOpenSettings(true)}
          onOpenChange={() => setOpenSettings(false)}
        />
      )}
    </div>
  );
};

export { BottomBar };
