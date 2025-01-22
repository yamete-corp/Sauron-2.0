use std::sync::Arc;
use chrono::Utc;
use loader_vars::types::{
    receive::{ ClientAction, ClientParams, RunConfigParams },
    send::{ LoaderSendPayload, ServerAction, ServerParams },
};
use tokio::{ net::TcpStream, sync::Mutex };
use anyhow::Result;
use crate::server::{
    http_server::{ Bot, LoaderInstance, ServerHandler },
    sanitization::verify_loader_get_config,
};

pub async fn route_loader(
    handler: &ServerHandler,
    payload: LoaderSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    match payload.action {
        ServerAction::GetConfig => { init(handler, payload, stream_ref).await }
    }
}

pub async fn init(
    handler: &ServerHandler,
    payload: LoaderSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    if let ServerParams::GetConfig(params) = payload.params {
        verify_loader_get_config(&params)?;

        let mut bot_map = handler.bot_map.write().await;
        let new_loader_instance = LoaderInstance {
            join_date: Utc::now().to_rfc3339(),
            version: payload.version,
            tag: params.tag,
        };
        if let Some(bot) = bot_map.get_mut(&payload.id) {
            bot.loader_instances.push(new_loader_instance);
            bot.verified = true;
        } else {
            bot_map.insert(
                payload.id.clone(),
                Bot::new(payload.id.clone(), vec![], vec![new_loader_instance], true)
            );
        }
        drop(bot_map);
        ServerHandler::send_action_to_loader(
            stream_ref,
            ClientAction::RunConfig,
            ClientParams::RunConfig(RunConfigParams::default())
        ).await
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
