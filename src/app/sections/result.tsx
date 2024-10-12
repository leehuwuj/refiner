"use client";

import React, { useContext, useEffect } from "react";
import {
  Tabs,
  Tab,
  Card,
  CardBody,
  Tooltip,
  Skeleton,
} from "@nextui-org/react";
import { LuClipboardCopy } from "react-icons/lu";
import { TranslateContext } from "../providers/translate";
import { MdClear } from "react-icons/md";
import { LanguageSelections } from "./language-selection";

const TextCard = ({
  title,
  content,
  isTranslating,
}: {
  title: string;
  content: string;
  isTranslating?: boolean;
}) => {
  const [copied, setCopied] = React.useState(false);
  const homeContext = useContext(TranslateContext);

  const handleCopy = (content: string) => {
    navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 2000);
  };

  return (
    <Card className="relative h-56 border-2 p-2" shadow="none">
      <CardBody>
        {content && content.length > 0 ? (
          <div className="absolute right-0 top-0 flex w-4 flex-col space-y-3">
            <button
              onClick={() =>
                homeContext.changeResult({ [title.toLowerCase()]: "" })
              }
            >
              <MdClear className="text-gray-500" size={18} />
            </button>
            <Tooltip isOpen={copied} content="Copied!">
              <button onClick={() => handleCopy(content)}>
                <LuClipboardCopy className="text-gray-500" size={18} />
              </button>
            </Tooltip>
          </div>
        ) : (
          <p className="text-sm text-gray-400">
            {/* Input the text and press <strong>Ctrl + Enter</strong> or <strong>Cmd + Enter</strong> to translate. */}
            Click chose a mode above that you want me to do: <br />-{" "}
            <strong>Translated</strong> to translate the text. <br />-{" "}
            <strong>Correct</strong> to correct grammar. <br />-{" "}
            <strong>Refine</strong> to refine the translation in a formal way.
          </p>
        )}
        {isTranslating && (
          <div className="space-y-3">
            <Skeleton className="w-3/5 rounded-lg">
              <div className="h-3 w-3/5 rounded-lg bg-default-200"></div>
            </Skeleton>
            <Skeleton className="w-4/5 rounded-lg">
              <div className="h-3 w-4/5 rounded-lg bg-default-200"></div>
            </Skeleton>
            <Skeleton className="w-2/5 rounded-lg">
              <div className="h-3 w-2/5 rounded-lg bg-default-300"></div>
            </Skeleton>
          </div>
        )}
        <p className="pr-5 text-sm">{content}</p>
      </CardBody>
    </Card>
  );
};

const Result = ({
  result,
  isTranslating,
  mode,
}: {
  result?: ResultTexts;
  isTranslating?: boolean;
  mode?: Mode;
}) => {
  const homeContext = useContext(TranslateContext);

  console.log("Result:", result);
  console.log("Mode", mode);

  // Result: { translate: 'Hello', correct: '', refine: '' }
  console.log("Text",);


  return (
    <div className="flex w-full flex-col">
      <LanguageSelections
        selectedLang={homeContext.languageConfig}
        changeLangConfig={homeContext.changeLangConfig}
      />
      <TextCard
        title={mode ?? "Default Title"}
        content={result?.[mode?.toLowerCase() as keyof typeof result] ?? ""}
        isTranslating={isTranslating}
      />
    </div>
  );
};

export { Result };
