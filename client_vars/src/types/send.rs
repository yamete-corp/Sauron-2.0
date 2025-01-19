use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoogeymanSendPayload {
    pub id: String,
    pub version: u64,
    pub action: ServerAction,
    pub params: ServerParams,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerAction {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerParams {}
