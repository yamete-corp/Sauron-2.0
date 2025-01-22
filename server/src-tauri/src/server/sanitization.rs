use anyhow::Result;
use anyhow::anyhow;
use client_vars::types::send::InitParams;
use client_vars::types::structs::DynamicInfo;
use client_vars::types::structs::HardwareInfo;
use client_vars::types::structs::OSInfo;
use loader_vars::types::send::GetConfigParams;

const MAX_VERSION: u64 = 2;
const MAX_TAG_LEN: usize = 16;
const DEVICE_ID_LEN: usize = 32;

pub fn verify_client_init(data: &InitParams) -> Result<()> {
    // Verify HardwareInfo
    verify_hardware_info(&data.bot_state.hw_info)?;

    // Verify OSInfo
    verify_os_info(&data.bot_state.os_info)?;

    // Verify DynamicInfo
    verify_dynamic_info(&data.bot_state.dynamic_info)?;

    // Check thumbnail length
    if let Some(thumbnail) = &data.bot_state.thumbnail {
        if thumbnail.len() > 1024 * 1024 {
            return Err(anyhow!("Thumbnail size exceeds 1MB"));
        }
    }
    Ok(())
}
// Verify HardwareInfo structure
fn verify_hardware_info(hw_info: &HardwareInfo) -> Result<()> {
    // Check cpu_brand length
    if hw_info.cpu_brand.len() > 64 {
        return Err(anyhow!("Invalid cpu_brand length"));
    }

    // Check cpu_vendor_id length
    if hw_info.cpu_vendor_id.len() > 64 {
        return Err(anyhow!("Invalid cpu_vendor_id length"));
    }

    // Check core_count
    if let Some(core_count) = hw_info.core_count {
        if core_count < 1 || core_count > 128 {
            return Err(anyhow!("Invalid core_count"));
        }
    }

    // Check total_ram
    if hw_info.total_ram < 1024 * 1024 * 1024 {
        return Err(anyhow!("Invalid total_ram"));
    }

    Ok(())
} // Verify OSInfo structure
fn verify_os_info(os_info: &OSInfo) -> Result<()> {
    // Check boot_time
    if os_info.boot_time < 1000000000 {
        return Err(anyhow!("Invalid boot_time"));
    }

    // Check host_name length
    if let Some(host_name) = &os_info.host_name {
        if host_name.len() > 64 {
            return Err(anyhow!("Invalid host_name length"));
        }
    }

    if let Some(os_version) = &os_info.os_version {
        if os_version.len() > 64 {
            return Err(anyhow!("Invalid os_version length"));
        }
    }

    // Check users length
    if os_info.users.len() > 20 {
        return Err(anyhow!("Invalid users length"));
    }

    Ok(())
}
pub fn verify_loader_get_config(data: &GetConfigParams) -> Result<()> {
    if data.tag.len() > MAX_TAG_LEN {
        return Err(anyhow!("Tag length exceeds maximum allowed length"));
    }
    Ok(())
}
pub fn verify_id_and_version(id: String, version: u64) -> Result<()> {
    if id.len() != DEVICE_ID_LEN {
        return Err(anyhow!("Device ID is not correct length"));
    }

    // Check version
    if version > MAX_VERSION {
        return Err(anyhow!("Version exceeds maximum version"));
    }
    Ok(())
}
fn verify_dynamic_info(dynamic_info: &DynamicInfo) -> Result<()> {
    // Check sysmem_uptime
    if dynamic_info.system_uptime < 1000000000 {
        return Err(anyhow!("Invalid system_uptime"));
    }

    // Check cpu_usage
    if dynamic_info.cpu_usage < 0.0 || dynamic_info.cpu_usage > 100.0 {
        return Err(anyhow!("Invalid cpu_usage"));
    }
    // Check active_window
    if dynamic_info.active_window.len() > 64 {
        return Err(anyhow!("Invalid active_window len()"));
    }
    Ok(())
}
// later verify bot loaded and verified
