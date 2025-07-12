import { Select, SelectItem } from "@heroui/react";
import { CiSettings } from "react-icons/ci";
import { Settings } from "./settings";
import React from "react";
import { useContext } from "react";
import { SettingContext } from "../providers/settings";
import { FaWandMagicSparkles } from "react-icons/fa6";

const BottomBar = () => {
  const [openSettings, setOpenSettings] = React.useState(false);
  const settingContext = useContext(SettingContext);

  return (
    <div className="fixed bottom-0 left-0 flex h-12 w-full items-center justify-between rounded-b-extra bg-default-100 pl-3 pr-3">
      <div className="flex space-x-3 w-full items-center align-middle justify-between">
        <button onClick={() => setOpenSettings(true)} className="text-default-500 hover:text-default-700">
          <CiSettings size={24} className="font-bold" />
        </button>
        <Select
          size="sm"
          className="w-4/12 text-xs"
          placeholder={settingContext.model || "Model"}
          value={settingContext.model || ""}
          onChange={(event) => settingContext.setModel(event.target.value)}
        >
          {settingContext.provider?.models?.map((model) => (
            <SelectItem key={model} id={model} className="text-xs">
              {model}
            </SelectItem>
          )) || []}
        </Select>
        <div className="relative w-full">
          <input
            className="rounded-md p-1.5 pl-8 text-sm w-full"
            placeholder="Writing style"
          >
          </input>
          <FaWandMagicSparkles size={18} className="absolute top-1.5 left-2 text-default-500" />
        </div>
        <p className="w-4/12 text-default-500 text-xs text-right">
          © 2025 Refiner App
        </p>
      </div>
      {
        openSettings && (
          <Settings
            isOpen={openSettings}
            onOpenChange={(isOpen) => setOpenSettings(isOpen)}
          />
        )
      }
    </div >

  );
};

export { BottomBar };
