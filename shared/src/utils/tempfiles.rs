use std::fs::File;
use std::io;
use std::os::windows::prelude::{ AsRawHandle, FromRawHandle };
use winapi::shared::ntdef::HANDLE;
use winapi::shared::minwindef::DWORD;
use winapi::um::fileapi::{ CreateFileA, CREATE_ALWAYS };
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::FILE_FLAG_DELETE_ON_CLOSE;
use winapi::um::winnt::{ FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_TEMPORARY, GENERIC_ALL };

pub fn create_temp_file() -> io::Result<File> {
    let file_name = "temp_file";
    let file_handle: HANDLE = unsafe {
        CreateFileA(
            file_name.as_ptr() as *const i8,
            GENERIC_ALL,
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

    Ok(unsafe { File::from_raw_handle(file_handle as std::os::windows::io::RawHandle) })
}

fn main() {
    match create_temp_file() {
        Ok(temp_file) => {
            println!("Temporary file created");
            // Use the temporary file...
            drop(temp_file)
        }
        Err(e) => println!("Error creating temporary file: {}", e),
    }
}
