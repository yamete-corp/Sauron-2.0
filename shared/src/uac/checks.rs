use std::mem;
use obfstr::obfstr as s;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::LPVOID;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::processthreadsapi::OpenProcessToken;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::TokenElevation;
use winapi::um::winnt::HANDLE;
use winapi::um::winnt::TOKEN_ELEVATION;
use winapi::um::winnt::TOKEN_QUERY;
use anyhow::{ Context, Result };
use super::impersonate_system::win32_error;

pub fn is_elevated() -> Result<bool> {
    // based on https://stackoverflow.com/a/8196291
    unsafe {
        let mut current_token_ptr: HANDLE = mem::zeroed();
        let mut token_elevation: TOKEN_ELEVATION = mem::zeroed();
        let token_elevation_type_ptr: *mut TOKEN_ELEVATION = &mut token_elevation;
        let mut size: DWORD = 0;

        let result = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut current_token_ptr);
        if result == 0 {
            return Err(win32_error()).context(s!("Failed to OpenProcessToken").to_string());
        }

        let result = GetTokenInformation(
            current_token_ptr,
            TokenElevation,
            token_elevation_type_ptr as LPVOID,
            mem::size_of::<winapi::um::winnt::TOKEN_ELEVATION_TYPE>() as u32,
            &mut size
        );
        if result == 0 {
            return Err(win32_error()).context(s!("Failed to GetTokenInformation").to_string());
        }

        return Ok(token_elevation.TokenIsElevated != 0);
    }
}

use std::os::windows::ffi::OsStringExt;
use winapi::um::winbase::GetUserNameW;

pub fn is_system() -> Result<bool> {
    unsafe {
        let size: usize = 1024;
        let mut buffer = vec![0u16; size];
        let size_ptr = &mut (size as u32);
        let result = GetUserNameW(buffer.as_mut_ptr(), size_ptr);
        if result == 0 {
            return Err(win32_error()).context(s!("Failed to GetUserNameW").to_string());
        }

        let username = std::ffi::OsString::from_wide(&buffer);
        let mut username_str = username.to_string_lossy().into_owned();
        username_str = username_str.trim_matches('\0').to_string();
        username_str = username_str.trim().to_string();
        username_str = username_str.trim_matches('\0').to_string();

        if username_str == "SYSTEM" {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
}
