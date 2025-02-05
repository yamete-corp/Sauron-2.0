use std::ptr::null_mut;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::shared::minwindef::FALSE;
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
use winapi::um::jobapi2::{
    AssignProcessToJobObject,
    CreateJobObjectW,
    SetInformationJobObject,
    TerminateJobObject,
};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::{
    JOBOBJECT_CPU_RATE_CONTROL_INFORMATION,
    JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
    JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
    PROCESS_ALL_ACCESS,
    JobObjectCpuRateControlInformation,
};
use anyhow::anyhow;
use obfstr::obfstr as s;

pub fn terminate_job(job_handle: *mut winapi::ctypes::c_void) {
    unsafe {
        TerminateJobObject(job_handle, 0);
    }
}

pub fn limit_cpu_usage(
    process_id: u32,
    cpu_limit_percent: u32,
    job_name: &str
) -> anyhow::Result<*mut winapi::ctypes::c_void> {
    let job_name = OsStr::new(job_name).encode_wide().chain(Some(0)).collect::<Vec<_>>();
    let job_handle = unsafe { CreateJobObjectW(null_mut(), job_name.as_ptr()) };
    if job_handle.is_null() {
        let error = unsafe { winapi::um::errhandlingapi::GetLastError() };
        if error != ERROR_ALREADY_EXISTS {
            return Err(
                anyhow!(format!("{}: {}", s!("Failed to create or open job object"), error))
            );
        }
    }

    let cpu_rate = cpu_limit_percent * 100;
    let mut cpu_info: JOBOBJECT_CPU_RATE_CONTROL_INFORMATION = unsafe { std::mem::zeroed() };
    cpu_info.ControlFlags =
        JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP;
    *(unsafe { cpu_info.u.CpuRate_mut() }) = cpu_rate; // Use the provided mutable accessor method

    let result = unsafe {
        SetInformationJobObject(
            job_handle,
            JobObjectCpuRateControlInformation,
            &mut cpu_info as *mut _ as *mut _,
            std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32
        )
    };

    if result == FALSE {
        return Err(
            anyhow!(
                format!("{}: {}", s!("Failed to set CPU rate control information"), unsafe {
                    winapi::um::errhandlingapi::GetLastError()
                })
            )
        );
    }

    let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, process_id) };
    if process_handle.is_null() {
        return Err(anyhow!(format!("{}: {}", s!("Failed to open process with ID"), process_id)));
    }

    let result = unsafe { AssignProcessToJobObject(job_handle, process_handle) };
    unsafe {
        CloseHandle(process_handle);
    }

    if result == FALSE {
        return Err(
            anyhow!(
                format!("{}: {}", s!("Failed to assign process to job object"), unsafe {
                    winapi::um::errhandlingapi::GetLastError()
                })
            )
        );
    }

    unsafe {
        CloseHandle(job_handle);
    }

    Ok(job_handle)
}
