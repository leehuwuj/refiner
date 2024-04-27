"use client";

import React, { useEffect } from "react";
import { Tabs, Tab, Card, CardBody, Tooltip } from "@nextui-org/react";
import { LuClipboardCopy } from "react-icons/lu";

const TextCard = ({ title, content }: { title: string; content: string }) => {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = (content: string) => {
    navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => {
      setCopied(false);
    }, 2000);
  };

  return (
    <Card className="relative h-56 p-2 border-2" shadow="none">
      <CardBody>
        {
          content && content.length > 0 ? (
            <Tooltip isOpen={copied} content="Copied!">
              <button
                className="absolute right-0 top-0"
                onClick={() => handleCopy(content)}
              >
                <LuClipboardCopy className="text-gray-500" size={18} />
              </button>
            </Tooltip>
          ) : <p className="text-gray-400 text-sm">
            {/* Input the text and press <strong>Ctrl + Enter</strong> or <strong>Cmd + Enter</strong> to translate. */}
            Click chose a mode above that you want me to do: <br />
            - <strong>Translated</strong> to translate the text. <br />
            - <strong>Refine</strong> to refine the translation. <br />
            - <strong>Refine (Formal)</strong> to refine the translation in a formal way.
          </p>
        }
        <p className="pr-5 text-sm">{content}</p>
      </CardBody>
    </Card >
  );
};

const Result = ({ result }: { result?: ResultTexts }) => {
  const data = [
    {
      title: "Translated",
      content: result?.translated ?? "",
    },
    {
      title: "Refine",
      content: result?.refine ?? "",
    },
    {
      title: "Refine (Formal)",
      content: result?.refineFormal ?? "",
    },
  ];

  useEffect(() => {
    console.log("Result:", result);
  }, [result]);

  return (
    <div className="flex w-full flex-col">
      <Tabs aria-label="Options">
        {data.map((item, index) => (
          <Tab key={index} title={item.title}>
            <TextCard title={item.title} content={item.content} />
          </Tab>
        ))}
      </Tabs>
    </div>
  );
};

export { Result };
