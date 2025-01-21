use std::sync::Arc;
use network::tor::{ ClientBotMap, TorServerHandler };
use lazy_static::lazy_static;
use tokio::sync::RwLock;
mod network;
mod sanitization;
use std::sync::mpsc;
use std::sync::{ Mutex };
lazy_static! {
    static ref CHANNEL: Arc<(mpsc::Sender<String>, Mutex<mpsc::Receiver<String>>)> = {
        let (tx, rx) = mpsc::channel();
        Arc::new((tx, Mutex::new(rx)))
    };
}
// lazy_static! {
//     pub static ref TOR_SERVER_HANDLER: Arc<RwLock<Option<TorServerHandler>>> = Arc::new(
//         RwLock::new(None)
//     );
// }

// #[tauri::command(async)]
// async fn get_client_map() -> Result<ClientBotMap, String> {
//     let server_ref = TOR_SERVER_HANDLER.read().await;
//     if let Some(tor_server) = server_ref.as_ref() {
//         return Ok(tor_server.client_bot_map.clone());
//     } else {
//         return Err("TOR_SERVER_HANDLER not initialized".to_string());
//     }
// }

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    // tokio_runtime.spawn(async move {
    //     if let Ok(mut tor_server) = TorServerHandler::new().await {
    //         let mut server_ref = TOR_SERVER_HANDLER.write().await;
    //         *server_ref = Some(tor_server.clone());
    //         drop(server_ref);

    //         if let Err(e) = tor_server.listen_for_connections().await {
    //             eprintln!("Server errored out: {}", e);
    //         }
    //     }
    // });
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
