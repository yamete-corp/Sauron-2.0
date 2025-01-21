use std::{
    fs::{ create_dir_all, File, OpenOptions },
    io::Write,
    os::windows::fs::MetadataExt,
    path::PathBuf,
};
use chrono::Utc;
use anyhow::{ Context, Result };
use crate::utils::encryption::{ convert_key_to_bytes, encrypt_timestamp, sauron_encrypt };
use obfstr::obfstr as s;

#[macro_export]
macro_rules! ref_log_internal {
    ($logger:expr, $level:expr, $s:expr) => {
        {
           
            $logger.log($level, obfstr::obfstr!($s));
            
        }
    };

    (
        $logger:expr,
        $level:expr,
        $fmt:expr,
        $($arg:tt)*
    ) => {
        {
            $logger.log($level, &format!("{}{:#?}", obfstr::obfstr!($fmt), $($arg)*));
            
        }
    };
}

#[macro_export]
macro_rules! ref_info {
    ($logger:expr, $s:expr) => {
        ref_log_internal!($logger, obfstr::obfstr!("INFO"), $s)
    };
    (
        $logger:expr,
        $fmt:expr,
        $($arg:tt)*
    ) => {
        ref_log_internal!($logger, obfstr::obfstr!("INFO"), $fmt, $($arg)*)
    };
}

#[macro_export]
macro_rules! ref_warn {
    ($logger:expr, $s:expr) => {
        ref_log_internal!($logger, obfstr::obfstr!("WARN"), $s)
    };
    (
        $logger:expr,
        $fmt:expr,
        $($arg:tt)*
    ) => {
        ref_log_internal!($logger, obfstr::obfstr!("WARN"), $fmt, $($arg)*)
    };
}

#[macro_export]
macro_rules! ref_err {
    ($logger:expr, $s:expr) => {
        ref_log_internal!($logger, obfstr::obfstr!("ERROR"), $s)
    };
    (
        $logger:expr,
        $fmt:expr,
        $($arg:tt)*
    ) => {
        ref_log_internal!($logger, obfstr::obfstr!("ERROR"), $fmt, $($arg)*)
    };
}

#[macro_export]
macro_rules! ref_verbose {
    ($logger:expr, $s:expr) => {
        ref_log_internal!($logger, obfstr::obfstr!("VERBOSE"), $s)
    };
    (
        $logger:expr,
        $fmt:expr,
        $($arg:tt)*
    ) => {
        ref_log_internal!($logger, obfstr::obfstr!("VERBOSE"), $fmt, $($arg)*)
    };
}

#[macro_export]
macro_rules! ref_tag {
    ($logger:expr, $s:expr) => {
        ref_log_internal!($logger, obfstr::obfstr!("TAG"), $s)
    };
}

#[derive(Debug, Clone)]
pub struct Logger {
    key: [u8; 32],
    log_file_path: PathBuf,
}

impl Logger {
    pub fn new(directory: PathBuf, encryption_key: String) -> Result<Self> {
        create_dir_all(&directory).context(s!("Failed to create log folder").to_string())?;

        let key: [u8; 32] = convert_key_to_bytes(&encryption_key);

        let time_now_log_name = encrypt_timestamp(key)?;

        let log_file_path = directory.join(time_now_log_name);
        File::create(&log_file_path).context(s!("Failed to create log file").to_string())?;

        let logger = Logger {
            key,
            log_file_path,
        };

        Ok(logger)
    }

    pub fn log(&self, level: &str, content: &str) {
        let file_size = self.get_file_size();
        if file_size > 5 * 1024 * 1024 {
            // 5 mb exceeded - end
            return;
        }
        let timestamp = Utc::now().to_rfc3339();
        let log_entry = format!("\n\n[{}] {}: {}", timestamp, level, content);

        if let Ok(data_to_write) = sauron_encrypt(self.key, log_entry.as_bytes()) {
            let _ = self.write_to_binary_file(&data_to_write);
        };
    }

    fn write_to_binary_file(&self, data: &[u8]) -> Result<()> {
        let mut file = OpenOptions::new().append(true).open(&self.log_file_path)?;
        file.write_all(data)?;
        file.write_all(b"\r\n\r\n")?;
        Ok(())
    }
    fn get_file_size(&self) -> u64 {
        std::fs
            ::metadata(&self.log_file_path)
            .map(|metadata| metadata.file_size())
            .unwrap_or(0)
    }
}
