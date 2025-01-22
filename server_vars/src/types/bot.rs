use serde::{ Deserialize, Serialize };
#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
}
