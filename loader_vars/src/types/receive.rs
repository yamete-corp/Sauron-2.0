use serde::{ Deserialize, Serialize };
use crate::constants::{
    hardcoded_client_fallback_source,
    hardcoded_client_version,
    loader_version,
};

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
pub enum FileSource {
    Url(String),
    Bytes(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunConfigParams {
    pub enabled: bool, // if disabled not even run
    pub client_source: Option<FileSource>,
    pub client_version: u64,
    pub self_source: Option<FileSource>,
    pub self_version_latest: u64,
    pub update_enabled: bool, // reinstall updated if version higher
}

impl Default for RunConfigParams {
    fn default() -> Self {
        Self {
            enabled: true,
            client_source: hardcoded_client_fallback_source(),
            client_version: hardcoded_client_version(),
            self_source: None,
            self_version_latest: loader_version(),
            update_enabled: false,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UninstallSelfParams {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSelfParams {}
