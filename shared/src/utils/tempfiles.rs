use std::fs::File;
use std::path::PathBuf;
use std::{ env, io };
use std::os::windows::prelude::{ AsRawHandle, FromRawHandle };
use winapi::shared::ntdef::HANDLE;
use winapi::shared::minwindef::DWORD;
use winapi::um::fileapi::{ CreateFileA, CREATE_ALWAYS };
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::FILE_FLAG_DELETE_ON_CLOSE;
use winapi::um::winnt::{ FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_TEMPORARY, GENERIC_WRITE };

pub fn create_temp_file(file_name: &str) -> io::Result<(File, PathBuf)> {
    let temp_dir = env::temp_dir();
    // file_name.push_str(&format!("{}", rand::random::<u64>()));
    let full_path = temp_dir.join(file_name);
    let full_path_str = full_path.to_str().unwrap();
    println!("{}", full_path_str);
    let file_handle: HANDLE = unsafe {
        CreateFileA(
            full_path_str.as_ptr() as *const i8,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_TEMPORARY | FILE_ATTRIBUTE_NORMAL | FILE_FLAG_DELETE_ON_CLOSE,
            std::ptr::null_mut()
        )
    };

    if file_handle == core::ptr::null_mut() {
        let error_code = unsafe { GetLastError() };
        return Err(io::Error::from_raw_os_error(error_code as i32));
    }

    let file = unsafe { File::from_raw_handle(file_handle as std::os::windows::io::RawHandle) };

    Ok((file, temp_dir))
}
