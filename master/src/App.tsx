import { useEffect, useRef, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { BotItem } from "./types";
import BotListItem from "./BotListItem";
import ContextMenu from "./ContextMenu";

function App() {
  const [loadedBots, setLoadedBots] = useState<{ [key: string]: BotItem }>({});
  const contextMenuRef = useRef<HTMLDivElement>(null);
  const [contextMenu, setContextMenu] = useState({
    visible: false,
    x: 0,
    y: 0,
    botId: "",
    botVersion: 0,
  });

  const hideContextMenu = () => {
    setContextMenu({ ...contextMenu, visible: false });
  };

  async function get_bots() {
    const response = await invoke("get_bot_items", { filterIds: [] });
    const botItemMap = JSON.parse(response as string);
    setLoadedBots(botItemMap);
    // console.log(loadedBots);
  }
  const handleBotMenu = (
    event: React.MouseEvent<HTMLDivElement>,
    botId: string,
    botVersion: number
  ) => {
    event.preventDefault();
    setContextMenu({
      visible: true,
      x: event.clientX,
      y: event.clientY,
      botId,
      botVersion,
    });
    console.log(`ContextMenu clicked on bot: ${botId}`);
  };
  useEffect(() => {
    setInterval(get_bots, 3000);
  }, []);
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (contextMenu.visible) {
        const contextMenuElement = document.querySelector(".context-menu");
        if (
          contextMenuElement &&
          !contextMenuElement.contains(event.target as Node)
        ) {
          hideContextMenu();
        }
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [contextMenu.visible]);
  return (
    <div className="container">
      <div className="bot-list">
        {Object.values(loadedBots).map((bot) => (
          <BotListItem key={bot.id} bot={bot} onClickMenu={handleBotMenu} />
        ))}
      </div>
      <ContextMenu
        x={contextMenu.x}
        y={contextMenu.y}
        visible={contextMenu.visible}
        botId={contextMenu.botId}
        botVersion={contextMenu.botVersion}
      />
    </div>
  );
}

export default App;
