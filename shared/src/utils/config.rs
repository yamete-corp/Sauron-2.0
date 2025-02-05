//! clean up

use std::fs::{ self, create_dir_all };
use serde::{ Deserialize, Serialize };
use crate::constants::{ configs_encryption_key, get_mib_config_path };
use super::{
    encryption::{ convert_key_to_bytes, sauron_decrypt, sauron_encrypt },
    functions::get_current_exe_dir,
};
use anyhow::Result;
use obfstr::obfstr as s;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HstCnfg {
    pub version: u64,
    pub folder_path: String,
    pub exe_path: String,
    pub config_path: String,
}

impl Default for HstCnfg {
    fn default() -> Self {
        Self {
            version: 0,
            folder_path: String::new(),
            exe_path: String::new(),
            config_path: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MibCnfg {
    pub cln_up_done: bool,
    pub ldrs: Vec<HstCnfg>,
    pub clnts: Vec<HstCnfg>,
}

impl Default for MibCnfg {
    fn default() -> Self {
        Self {
            cln_up_done: false,
            ldrs: Vec::new(),
            clnts: Vec::new(),
        }
    }
}

pub fn load_mib_config(try_create: bool) -> Result<MibCnfg> {
    // info!("loading mib config");
    let file_path = get_mib_config_path();
    let default_config = MibCnfg::default();
    // info!("file_path mib config: ", file_path);
    let enc_key_bytes = convert_key_to_bytes(&configs_encryption_key());
    if let Ok(encrypted_data) = fs::read(file_path.clone()) {
        if let Ok(decrypted_data) = sauron_decrypt(enc_key_bytes, &encrypted_data) {
            if let Ok(cnfg) = serde_json::from_slice(&decrypted_data) {
                return Ok(cnfg);
            }
        }
    }
    if try_create {
        let encrypted_data = sauron_encrypt(enc_key_bytes, &serde_json::to_vec(&default_config)?)?;
        create_dir_all(file_path.parent().unwrap())?;
        fs::write(file_path, encrypted_data)?;
    }

    Ok(default_config)
}
pub fn write_mib_config(config: &MibCnfg) -> Result<()> {
    // info!("writing mib config: ", config);
    let file_path = get_mib_config_path();
    let enc_key_bytes = convert_key_to_bytes(&configs_encryption_key());
    let encrypted_data = sauron_encrypt(enc_key_bytes, &serde_json::to_vec(config)?)?;
    create_dir_all(file_path.parent().unwrap())?;
    fs::write(file_path, encrypted_data)?;
    Ok(())
}
