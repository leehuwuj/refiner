import dynamic from "next/dynamic";
import { Providers } from "../provider";

const SettingsClient = dynamic(() => import("./settings-client"));

export default function SettingsPage() {
  return (
    <Providers>
      <SettingsClient />
    </Providers>
  );
}
