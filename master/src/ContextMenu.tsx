// ContextMenu.tsx
import React, { MouseEventHandler } from "react";
import "./ContextMenu.css";
import { event } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/tauri";
import { WebviewWindow } from "@tauri-apps/api/window";

interface ContextMenuItem {
  label: string;
  onClick: () => void;
}

interface ContextMenuProps {
  x: number;
  y: number;
  visible: boolean;
  botId: string;
  botVersion: number;
}

const ContextMenu: React.FC<ContextMenuProps> = ({
  x,
  y,
  visible,
  botId,
  botVersion,
}) => {
  if (!visible) return null;
  const items = [
    {
      label: "Console",
      onClick: async () => {
        console.log(botId);
        const webview = new WebviewWindow("theUniqueLabel", {
          url: "console.html",
          title: botId + ":" + botVersion,
        });
        webview.once("tauri://created", function () {
          // webview window successfully created
          console.log(botId);
        });
        webview.once("tauri://error", function (e) {
          console.log(e);
          // an error occurred during webview window creation
        });
        // await invoke("open_console", {
        //   botId: botId,
        // });
      },
    },
    {
      label: "Test",
      onClick: () => console.log("test clicked"),
    },
  ];
  return (
    <div className="context-menu" style={{ top: y, left: x }}>
      {items.map((item, index) => (
        <div
          key={index}
          className="context-menu-item"
          onClick={(e) => {
            e.preventDefault();
            item.onClick();
          }}
          onContextMenu={(e) => {
            e.preventDefault();
            item.onClick();
          }}
        >
          {item.label}
        </div>
      ))}
    </div>
  );
};

export default ContextMenu;
