use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LdrSnd {
    pub id: String,
    pub version: u64,
    pub action: SrvAct,
    pub params: SrvPrms,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SrvAct {
    GtCnfg,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SrvPrms {
    GtCnfg(GtCnfgPrms),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GtCnfgPrms {
    pub tag: String,
}
