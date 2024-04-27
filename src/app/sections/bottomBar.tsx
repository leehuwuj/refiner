import { Select, SelectItem } from "@nextui-org/select";
import { CiSettings } from "react-icons/ci";
import { Settings } from "./settings";
import React from "react";
import { useContext } from "react";
import { SettingContext } from "../providers/settings";

const BottomBar = () => {
  const [openSettings, setOpenSettings] = React.useState(false);
  const settingContext = useContext(SettingContext);

  const updateModel = (model: string) => {
    if (settingContext) {
      settingContext.setModel(model);
    }
  };

  return (
    <div className="fixed bottom-0 left-0 flex h-12 w-full items-center justify-between rounded-b-extra bg-gray-200 pl-3 pr-3">
      <div className="flex items-center space-x-3">
        <button onClick={() => setOpenSettings(true)} className="text-gray-500">
          <CiSettings size={24} className="font-bold" />
        </button>
        <Select
          size="sm"
          placeholder="Llama 3"
          value="llama3"
          className="w-40"
          onChange={(event) => updateModel(event.target.value)}
        >
          <SelectItem value="phi3" key="phi3">
            Phi 3
          </SelectItem>
          <SelectItem value="llama3" key="llama3">
            Llama 3
          </SelectItem>
        </Select>
      </div>
      <p className="flex items-center text-sm text-gray-500">
        © 2024 Refiner App, leehuwuj.
      </p>
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
