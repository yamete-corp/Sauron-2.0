use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotState {
    pub ip_info: IpInfo,
    pub hw_info: HardwareInfo,
    pub os_info: OSInfo,
    pub dynamic_info: DynamicInfo,
    pub thumbnail: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HardwareInfo {
    pub cpu_brand: String,
    pub cpu_vendor_id: String,
    pub core_count: Option<usize>,
    pub total_ram: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OSInfo {
    pub boot_time: u64,
    pub host_name: Option<String>,
    pub os_version: Option<String>,
    pub users: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicInfo {
    pub system_uptime: u64,
    pub cpu_usage: f32,
    pub free_ram: u64,
    pub active_window: String,
}

// camel case because this is the output we get
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IpInfo {
    pub query: String, // ip
    pub continent_code: String,
    pub continent: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub region_name: String,
    pub city: String,
    pub district: String,
    pub zip: String,
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    pub offset: i32,
    pub currency: String,
    pub isp: String,
    pub org: String,
    pub asname: String,
    pub mobile: bool,
    pub proxy: bool,
    pub hosting: bool,
}
impl Default for IpInfo {
    fn default() -> Self {
        IpInfo {
            query: String::new(),
            continent_code: String::new(),
            continent: String::new(),
            country: String::new(),
            country_code: String::new(),
            region: String::new(),
            region_name: String::new(),
            city: String::new(),
            district: String::new(),
            zip: String::new(),
            lat: 0.0,
            lon: 0.0,
            timezone: String::new(),
            offset: 0,
            currency: String::new(),
            isp: String::new(),
            org: String::new(),
            asname: String::new(),
            mobile: false,
            proxy: false,
            hosting: false,
        }
    }
}
