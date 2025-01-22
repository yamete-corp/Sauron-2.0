use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderSendPayload {
    pub id: String,
    pub version: u64,
    pub action: ServerAction,
    pub params: ServerParams,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerAction {
    GetConfig,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerParams {
    GetConfig(GetConfigParams),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetConfigParams {
    pub tag: String,
}
