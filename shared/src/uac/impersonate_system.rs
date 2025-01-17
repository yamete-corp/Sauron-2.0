use anyhow::{ anyhow, Context, Result };
use obfstr::obfstr as s;
use std::ffi::{ OsStr, OsString };
use std::os::windows::ffi::{ OsStrExt, OsStringExt };
use std::ptr::null_mut;
use std::thread;
use std::time::Duration;
use winapi::{
    shared::{
        minwindef::{ BOOL, DWORD, FALSE, LPBYTE, LPVOID },
        ntdef::{ HANDLE, LPCWSTR, LPWSTR, NULL },
    },
    um::{
        errhandlingapi::GetLastError,
        handleapi::{ CloseHandle, INVALID_HANDLE_VALUE },
        minwinbase::SECURITY_ATTRIBUTES,
        processthreadsapi::{ OpenProcess, OpenProcessToken, PROCESS_INFORMATION, STARTUPINFOW },
        securitybaseapi::{ DuplicateTokenEx, ImpersonateLoggedOnUser },
        tlhelp32::{
            CreateToolhelp32Snapshot,
            Process32First,
            Process32Next,
            PROCESSENTRY32,
            TH32CS_SNAPPROCESS,
        },
        winbase::{ FormatMessageW, CREATE_UNICODE_ENVIRONMENT, STARTF_USESHOWWINDOW },
        winnt::{
            SecurityImpersonation,
            TokenImpersonation,
            MAXIMUM_ALLOWED,
            TOKEN_ALL_ACCESS,
            TOKEN_DUPLICATE,
        },
        winsvc::{
            CloseServiceHandle,
            OpenSCManagerA,
            OpenServiceA,
            QueryServiceStatusEx,
            StartServiceA,
            SC_MANAGER_CONNECT,
            SC_STATUS_PROCESS_INFO,
            SERVICE_QUERY_STATUS,
            SERVICE_RUNNING,
            SERVICE_START,
            SERVICE_STATUS_PROCESS,
            SERVICE_STOPPED,
        },
        winuser::{ SW_HIDE, SW_SHOW },
    },
};

pub const LOGON_WITH_PROFILE: DWORD = 0x00000001;

extern "system" {
    pub fn CreateProcessWithTokenW(
        hToken: HANDLE,
        dwLogonFlags: DWORD,
        lpApplicationName: LPCWSTR,
        lpCommandLine: LPWSTR,
        dwCreationFlags: DWORD,
        lpEnvironment: LPVOID,
        lpCurrentDirectory: LPCWSTR,
        lpStartupInfo: *mut STARTUPINFOW,
        lpProcessInformation: *mut PROCESS_INFORMATION
    ) -> BOOL;
}

pub fn format_error_message(error_code: DWORD) -> String {
    let mut buffer: [u16; 256] = [0; 256];
    unsafe {
        FormatMessageW(
            winapi::um::winbase::FORMAT_MESSAGE_FROM_SYSTEM,
            NULL as _,
            error_code,
            0,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            NULL as _
        );
    }
    String::from_utf16_lossy(&buffer).trim().to_string()
}

pub fn win32_error() -> anyhow::Error {
    anyhow!(format_error_message(unsafe { GetLastError() }))
}

pub fn execute_file_as_system(app_name: &str, cmd_args: &str, visible: bool) -> Result<()> {
    // info!("Executing file as TrustedInstaller: ", &app_name);
    // info!("args: ", &cmd_args);
    // info!("visible: ", &visible);

    impersonate_as_system().context(s!("Failed to impersonate as SYSTEM").to_string())?;

    let pid = start_ti_service_and_get_pid().context(
        s!("Failed to start TrustedInstaller service").to_string()
    )?;

    let token = create_access_token_from_pid(pid).context(
        s!("Failed to create access token from TrustedInstaller PID").to_string()
    )?;

    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = (if visible { SW_SHOW } else { SW_HIDE }) as u16;

    let full_command_str = if cmd_args.is_empty() {
        app_name
    } else {
        &format!("{} {}", app_name, cmd_args)
    };
    let full_command = OsStr::new(full_command_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    // Ensure token is always closed
    let result = unsafe {
        let success =
            CreateProcessWithTokenW(
                token,
                LOGON_WITH_PROFILE,
                null_mut(),
                full_command.as_ptr() as *mut u16,
                CREATE_UNICODE_ENVIRONMENT,
                null_mut(),
                null_mut(),
                &mut si,
                &mut pi
            ) != 0;

        CloseHandle(token);

        if !success {
            // Close process and thread handles
            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
            Err(win32_error())
        } else {
            // Close process and thread handles
            CloseHandle(pi.hProcess);
            CloseHandle(pi.hThread);
            Ok(())
        }
    };

    result.context(s!("Failed to CreateProcessWithTokenW").to_string())
}

pub fn create_access_token_from_pid(process_id: u32) -> Result<HANDLE> {
    let process_handle = unsafe { OpenProcess(MAXIMUM_ALLOWED, FALSE, process_id) };
    if process_handle == INVALID_HANDLE_VALUE {
        return Err(win32_error()).context(s!("Failed to open process").to_string());
    }

    // Ensure process handle is closed on all code paths
    let result = unsafe {
        let mut token_handle = INVALID_HANDLE_VALUE;
        if OpenProcessToken(process_handle, TOKEN_DUPLICATE, &mut token_handle) == 0 {
            return Err(win32_error()).context(s!("Failed to open process token").to_string());
        }

        let mut new_token_handle = INVALID_HANDLE_VALUE;
        let mut sec_attr = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: FALSE,
        };

        let success =
            DuplicateTokenEx(
                token_handle,
                TOKEN_ALL_ACCESS,
                &mut sec_attr,
                SecurityImpersonation,
                TokenImpersonation,
                &mut new_token_handle
            ) != 0;

        // Clean up handles
        CloseHandle(token_handle);

        if !success {
            Err(win32_error()).context(s!("Failed to duplicate token").to_string())
        } else {
            Ok(new_token_handle)
        }
    };

    unsafe {
        CloseHandle(process_handle);
    }
    result
}

pub fn get_pid_from_process_name(process_name: &str) -> Result<DWORD> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(win32_error()).context(s!("Failed to create process snapshot").to_string());
    }

    let result = unsafe {
        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as DWORD;

        if Process32First(snapshot, &mut entry) == 0 {
            return Err(win32_error()).context(s!("Failed to get first process").to_string());
        }

        let mut found_pid = None;
        loop {
            let sz_exe_file_len = entry.szExeFile
                .iter()
                .position(|&r| r == 0)
                .context(s!("Failed to find process name terminator").to_string())?;

            let current_process_name = OsString::from_wide(
                &entry.szExeFile[0..sz_exe_file_len]
                    .iter()
                    .map(|&c| c as u16)
                    .collect::<Vec<u16>>()
            );

            if current_process_name == OsString::from(process_name) {
                found_pid = Some(entry.th32ProcessID);
                break;
            }

            if Process32Next(snapshot, &mut entry) == 0 {
                break;
            }
        }

        found_pid.ok_or_else(|| anyhow!("{}{}", s!("Process not found: "), process_name))
    };

    unsafe {
        CloseHandle(snapshot);
    }
    result
}

pub fn impersonate_as_system() -> Result<()> {
    let pid = get_pid_from_process_name(s!("winlogon.exe")).context(
        s!("Failed to get winlogon.exe PID").to_string()
    )?;

    let token = create_access_token_from_pid(pid).context(
        s!("Failed to create access token").to_string()
    )?;

    let result = unsafe {
        let success = ImpersonateLoggedOnUser(token) != 0;
        CloseHandle(token);

        if !success {
            Err(win32_error())
        } else {
            Ok(())
        }
    };

    result.context(s!("Failed to impersonate logged-on user").to_string())
}

pub fn start_ti_service_and_get_pid() -> Result<DWORD> {
    const SLEEP_INTERVAL: DWORD = 50;
    const MAX_ATTEMPTS: DWORD = 100; // 5 seconds total

    let scm = unsafe { OpenSCManagerA(null_mut(), null_mut(), SC_MANAGER_CONNECT) };
    if scm.is_null() {
        return Err(win32_error()).context(s!("Failed to open Service Control Manager").to_string());
    }

    let result = unsafe {
        let service = OpenServiceA(
            scm,
            s!("TrustedInstaller\0").as_ptr() as *const i8,
            SERVICE_START | SERVICE_QUERY_STATUS
        );

        if service.is_null() {
            CloseServiceHandle(scm);
            return Err(win32_error()).context(
                s!("Failed to open TrustedInstaller service").to_string()
            );
        }

        let mut pid = 0;
        let mut attempts = 0;

        while attempts < MAX_ATTEMPTS {
            let mut ssp: SERVICE_STATUS_PROCESS = std::mem::zeroed();
            let mut bytes_needed = 0;

            if
                QueryServiceStatusEx(
                    service,
                    SC_STATUS_PROCESS_INFO,
                    &mut ssp as *mut _ as LPBYTE,
                    std::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
                    &mut bytes_needed
                ) == 0
            {
                let err = win32_error();
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                return Err(err).context(s!("Failed to query service status").to_string());
            }

            match ssp.dwCurrentState {
                SERVICE_RUNNING => {
                    pid = ssp.dwProcessId;
                    break;
                }
                SERVICE_STOPPED => {
                    if StartServiceA(service, 0, null_mut()) == 0 {
                        let err = win32_error();
                        CloseServiceHandle(service);
                        CloseServiceHandle(scm);
                        return Err(err).context(s!("Failed to start service").to_string());
                    }
                }
                _ => {}
            }

            thread::sleep(Duration::from_millis(SLEEP_INTERVAL as u64));
            attempts += 1;
        }

        CloseServiceHandle(service);
        CloseServiceHandle(scm);

        if pid == 0 {
            Err(
                anyhow!(
                    s!(
                        "5 seconds Timeout waiting for TrustedInstaller service to start"
                    ).to_string()
                )
            )
        } else {
            Ok(pid)
        }
    };

    result
}
