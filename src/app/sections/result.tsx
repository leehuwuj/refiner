"use client";

import React, { useEffect } from "react";
import { Tabs, Tab, Card, CardBody, Tooltip } from "@nextui-org/react";

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
    <Card>
      <CardBody>
        <div className="relative h-40">
          <Tooltip isOpen={copied} content="Copied!">
            <button
              className="absolute right-0 top-0"
              onClick={() => handleCopy(content)}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-6 w-6"
                viewBox="0 0 24 24"
              >
                <path d="M0 0h24v24H0z" fill="none" />
                <path d="M18 9H6v2h12V9zm0-4H6v2h12V5zm0 8H6v2h12v-2zm-3 4H6v2h9v-2z" />
              </svg>
            </button>
          </Tooltip>
          <p className="pr-5 text-sm">{content}</p>
        </div>
      </CardBody>
    </Card>
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
