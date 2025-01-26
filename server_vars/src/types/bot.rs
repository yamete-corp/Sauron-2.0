use serde::{ Deserialize, Serialize };
use shared::utils::config::MibCnfg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BotItem {
    pub id: String,
    pub flag: String,
    pub name: String,
    pub cpu_brand: String,
    pub ram: String,
    pub ping: String,
    pub join_date: String,
    pub system_boot_time: u64,
    pub region: String,
    pub os_info: String,
    pub active_window: String,
    pub mib_config: Option<MibCnfg>,
    pub client_instances: Vec<FrontendClientInstance>,
    pub loader_instances: Vec<FrontendLoaderInstance>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FrontendClientInstance {
    pub join_date: String,
    pub version: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FrontendLoaderInstance {
    pub join_date: String,
    pub version: u64,
    pub tag: String,
}
