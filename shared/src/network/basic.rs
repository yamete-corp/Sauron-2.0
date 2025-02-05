use anyhow::Result;
use std::io::Read;

pub fn download_bytes_from_url(url: &str) -> Result<Vec<u8>> {
    let reader = ureq::get(url).call()?.into_reader();
    let mut reader = std::io::BufReader::new(reader);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}
pub fn download_file_from_url_router_pastebin(url: &str) -> Result<Vec<u8>> {
    // Download the URL to get the file URL
    let response = ureq::get(url).call()?;
    let file_url = response.into_string()?;

    // Download the file from the file URL
    let bytes = download_bytes_from_url(&file_url)?;
    Ok(bytes)
}
