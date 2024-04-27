"use client";

import React from "react";
import { Textarea } from "@nextui-org/react";

const TextInput = ({ changeText }: { changeText: (text: string) => void }) => {
  return (
    <Textarea
      variant="bordered"
      placeholder="Input your text here to see the magic happen!"
      disableAutosize
      classNames={{
        input: "h-40", // Increase the height here
      }}
      onChange={(e) => changeText(e.target.value)}
      aria-label="Input text area"
    />
  );
};

export { TextInput };
