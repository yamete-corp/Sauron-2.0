import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { BotItem } from "./types";
import BotListItem from "./BotListItem";

function App() {
  const [loadedBots, setLoadedBots] = useState<{ [key: string]: BotItem }>({});
  async function get_bots() {
    const response = await invoke("get_bot_items", { filterIds: [] });
    const botItemMap = JSON.parse(response as string);
    setLoadedBots(botItemMap);
    console.log(loadedBots);
  }
  const handleBotMenu = (
    event: React.MouseEvent<HTMLDivElement>,
    botId: string
  ) => {
    event.preventDefault();
    console.log(`ContextMenu clicked on bot: ${botId}`);
    // Implement your context menu logic here
  };
  return (
    <div className="container">
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          get_bots();
        }}
      >
        <input
          id="greet-input"
          // onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      {Object.values(loadedBots).map((bot) => (
        <BotListItem key={bot.id} bot={bot} onClickMenu={handleBotMenu} />
      ))}
    </div>
  );
}

export default App;
