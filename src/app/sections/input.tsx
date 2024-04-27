"use client";

import React from "react";
import { Textarea } from "@nextui-org/react";

const TextInput = ({ changeText }: { changeText: (text: string) => void }) => {
  return (
    <Textarea
      variant="bordered"
      disableAutosize
      placeholder="Input your text here and press Cmd + Enter to see the magic happen!"
      onChange={(e) => changeText(e.target.value)}
      aria-label="Input text area"
      classNames={{
        base: "h-56",
        input: "h-56",
        inputWrapper: "",
      }}
    />
  );
};

export { TextInput };
