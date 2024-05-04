"use client";

import React from "react";
import { Textarea } from "@nextui-org/react";

const TextInput = ({
  inputText,
  changeText,
}: {
  inputText?: string;
  changeText: (text: string) => void;
}) => {
  return (
    <Textarea
      id="input-text"
      variant="bordered"
      disableAutosize
      placeholder="The inputted language is automatically detected. Enter your text here and press Cmd + Enter to see the magic happen!"
      value={inputText}
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
