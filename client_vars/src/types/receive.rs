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
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientParams {
    UninstallSelf(UninstallSelfParams),
    UpdateSelf(UpdateSelfParams),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UninstallSelfParams {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSelfParams {}
