import React, { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./ConsolePage.css";

function ConsolePage() {
  // State to store the console output
  const [consoleOutput, setConsoleOutput] = useState("");
  // State to store the current command
  const [command, setCommand] = useState("");
  // Reference to the console output div
  const consoleOutputRef = useRef<HTMLDivElement>(null);

  // Function to execute a command and update the console output
  const executeCommand = async () => {
    try {
      const titleParts = document.title.split(":");
      const botId = titleParts[0];
      const botVersion = titleParts[1];
      const output = await invoke("send_console_command", {
        botId,
        botVersion,
        command,
      });

      setConsoleOutput(output as string);
    } catch (error) {
      console.error(error);
    }
    setCommand("");
  };

  // Function to handle key press
  const handleKeyPress = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      executeCommand();
    }
  };

  useEffect(() => {
    // Scroll the console output to the bottom when it changes
    if (consoleOutputRef.current) {
      consoleOutputRef.current.scrollTop =
        consoleOutputRef.current.scrollHeight;
    }
  }, [consoleOutput]);

  return (
    <div className="container">
      <div ref={consoleOutputRef} className="console-output">
        {consoleOutput}
      </div>
      <div className="input-container">
        <input
          type="text"
          value={command}
          onChange={(event) => setCommand(event.target.value)}
          onKeyUp={handleKeyPress}
        />
        <button type="button" onClick={executeCommand}>
          Execute
        </button>
      </div>
    </div>
  );
}

export default ConsolePage;
