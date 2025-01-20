use anyhow::Result;
use std::io::Read;

pub fn download_bytes_from_url(url: &str) -> Result<Vec<u8>> {
    let reader = ureq::get(url).call()?.into_reader();
    let mut reader = std::io::BufReader::new(reader);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}
