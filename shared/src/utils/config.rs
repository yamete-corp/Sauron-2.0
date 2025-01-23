//! clean up

use std::fs;
use serde::{ Deserialize, Serialize };
use crate::constants::{ mib_config_encryption_key, get_mib_config_path };
use super::encryption::{ convert_key_to_bytes, sauron_decrypt, sauron_encrypt };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HstCnfg {
    pub version: u64,
    pub folder_path: String,
    pub exe_path: String,
    pub config_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub fn load_mib_config(try_create: bool) -> MibCnfg {
    // info!("loading mib config");
    let file_path = get_mib_config_path();
    let default_config = MibCnfg::default();
    // info!("file_path mib config: ", file_path);

    let enc_key_bytes = convert_key_to_bytes(&mib_config_encryption_key());
    if !file_path.exists() {
        if try_create {
            if
                let Ok(encrypted_data) = sauron_encrypt(
                    enc_key_bytes,
                    &serde_json::to_vec(&default_config).unwrap()
                )
            {
                match fs::write(file_path, encrypted_data) {
                    Ok(()) => {} //info!("wrote default mib config: ", default_config),
                    Err(_error) => {} //err!("failed to write mib config: ", error),
                }
            }
        }

        return default_config;
    }

    if let Ok(encrypted_data) = fs::read(file_path.clone()) {
        if let Ok(decrypted_data) = sauron_decrypt(enc_key_bytes, &encrypted_data) {
            match serde_json::from_slice(&decrypted_data) {
                Ok(config) => {
                    // info!("loaded mib config: ", config);
                    return config;
                }
                Err(_error) => {
                    // err!("Error parsing mib.xml overriding with default: ", error);
                    if
                        let Ok(encrypted_data) = sauron_encrypt(
                            enc_key_bytes,
                            &serde_json::to_vec(&default_config).unwrap()
                        )
                    {
                        match fs::write(file_path, encrypted_data) {
                            Ok(()) => {} //info!("wrote default mib config: ", default_config),
                            Err(_error) => {} //err!("failed to write mib config: ", error),
                        }
                    }
                }
            }
        }
    }
    default_config
}

pub fn write_mib_config(config: &MibCnfg) {
    // info!("writing mib config: ", config);

    let file_path = get_mib_config_path();
    let enc_key_bytes = convert_key_to_bytes(&mib_config_encryption_key());
    let encrypted_data = sauron_encrypt(
        enc_key_bytes,
        &serde_json::to_vec(config).unwrap()
    ).unwrap();
    fs::write(file_path, encrypted_data).unwrap();
}
