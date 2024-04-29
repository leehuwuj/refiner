import { Select, SelectItem } from "@nextui-org/select";
import { CiSettings } from "react-icons/ci";
import { Settings } from "./settings";
import React from "react";
import { useContext } from "react";
import { SettingContext } from "../providers/settings";

const BottomBar = () => {
  const [openSettings, setOpenSettings] = React.useState(false);
  const settingContext = useContext(SettingContext);

  return (
    <div className="fixed bottom-0 left-0 flex h-12 w-full items-center justify-between rounded-b-extra bg-gray-200 pl-3 pr-3">
      <div className="flex items-center space-x-3">
        <button onClick={() => setOpenSettings(true)} className="text-gray-500">
          <CiSettings size={24} className="font-bold" />
        </button>
        <Select
          size="sm"
          className="w-40"
          placeholder={settingContext.model || "Select a model"}
          value={settingContext.model || ""}
          onChange={(event) => settingContext.setModel(event.target.value)}
        >
          {
            settingContext.provider?.models?.map((model) => (
              <SelectItem value={model} key={model} id={model}>
                {model}
              </SelectItem>
            )) || []
          }
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
