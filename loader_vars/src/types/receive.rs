use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderReceivePayload {
    pub action: ClientAction,
    pub params: ClientParams,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientAction {
    RunConfig,
    UninstallSelf,
    UpdateSelf,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientParams {
    RunConfig(RunConfigParams),
    UninstallSelf(UninstallSelfParams),
    UpdateSelf(UpdateSelfParams),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunConfigParams {
    pub enabled: bool, // if disabled not even run
    pub client_bytes: Option<Vec<u8>>,
    pub client_url: Option<String>,
    pub client_version: u64,
    pub self_bytes: Option<Vec<u8>>,
    pub self_url: Option<String>,
    pub self_version_latest: u64,
    pub update_enabled: bool, // reinstall updated if version higher
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UninstallSelfParams {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSelfParams {}
