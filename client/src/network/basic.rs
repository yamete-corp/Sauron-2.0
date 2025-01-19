use std::path::PathBuf;
use anyhow::Result;
use obfstr::obfstr as s;
use client_vars::types::structs::IpInfo;

pub async fn get_ip_info() -> Result<IpInfo> {
    let res = reqwest::get(s!("http://ip-api.com/json?fields=66791423")).await?;
    let text = res.text().await?;
    let ip_info: IpInfo = serde_json::from_str(&text)?;
    Ok(ip_info)
}
pub async fn download_file_to_path(url: &str, path: PathBuf) -> Result<()> {
    let res = reqwest::get(url).await?;
    if !res.status().is_success() {
        return Err(anyhow::Error::msg(s!("Failed to download file").to_owned()));
    }
    if !path.exists() || !path.is_file() {
        tokio::fs::File::create(&path).await?;
    }

    let bytes = res.bytes().await?.to_vec();
    tokio::fs::write(path, bytes).await?;
    Ok(())
}
