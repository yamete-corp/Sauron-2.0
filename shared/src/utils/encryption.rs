use base64::{ engine::general_purpose, Engine as _ };
use chacha20poly1305::{ ChaCha20Poly1305, Key, KeyInit, Nonce };
use chrono::Utc;
use chacha20poly1305::aead::Aead;
use rand::{ thread_rng, Rng };
use anyhow::{ anyhow, Context, Result };
use obfstr::obfstr as s;

pub fn sauron_encrypt(key: [u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    let nonce = generate_nonce();

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));

    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|e| anyhow!(format!("{}{}", s!("Failed to sauron_encrypt: "), e)))?;
    let mut encrypted_data_with_nonce = Vec::new();
    encrypted_data_with_nonce.extend_from_slice(&encrypted);
    encrypted_data_with_nonce.extend_from_slice(&nonce);
    Ok(encrypted_data_with_nonce)
}
pub fn sauron_decrypt(key: [u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() <= 12 {
        return Err(anyhow!(s!("Data is too short to contain a nonce").to_owned()));
    }
    let nonce = &data[data.len() - 12..];
    let encrypted_data = &data[..data.len() - 12];
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));

    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), encrypted_data)
        .map_err(|e| anyhow!(format!("{}{}", s!("Failed to sauron_decrypt: "), e)))?;
    Ok(decrypted)
}

pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    thread_rng().fill(&mut nonce);
    nonce
}

pub fn convert_key_to_bytes(key: &str) -> [u8; 32] {
    let mut key_bytes = key.as_bytes().to_vec();
    key_bytes.resize(32, 0);

    let mut array = [0u8; 32];
    array.copy_from_slice(&key_bytes);

    array
}
pub fn convert_bytes_to_key(key_bytes: [u8; 32]) -> Result<String> {
    let mut trimmed_bytes = key_bytes.to_vec();
    while trimmed_bytes.last().context(s!("No more bytes to trim").to_string())? == &0 {
        trimmed_bytes.truncate(trimmed_bytes.len() - 1);
    }
    let key_string = String::from_utf8(trimmed_bytes)?;
    Ok(key_string)
}

pub fn encrypt_timestamp(encryption_key: [u8; 32]) -> Result<String> {
    let now = Utc::now().timestamp();
    let encrypted_datetime = sauron_encrypt(encryption_key, &now.to_le_bytes())?;

    Ok(general_purpose::STANDARD.encode(encrypted_datetime).replace('/', "-").replace('+', "_"))
}

pub fn decrypt_timestamp(encrypted_string: String, encryption_key: [u8; 32]) -> Result<i64> {
    let encrypted_data = general_purpose::STANDARD
        .decode(encrypted_string.replace('-', "/").replace('_', "+"))
        .unwrap();

    // Decrypt the datetime
    let decrypted_datetime = sauron_decrypt(encryption_key, &encrypted_data)?;
    let byte_array: [u8; 8] = decrypted_datetime
        .into_iter()
        .collect::<Vec<u8>>()
        .try_into()
        .map_err(|_| anyhow!(s!("Failed to convert to byte array").to_owned()))?;

    // Return the decrypted datetime as a string
    Ok(i64::from_le_bytes(byte_array))
}
