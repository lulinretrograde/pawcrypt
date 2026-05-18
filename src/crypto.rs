use crate::volume::{NONCE_LEN, SALT_LEN};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use anyhow::{bail, Result};
use argon2::{Argon2, Params, Algorithm, Version};
use zeroize::Zeroize;

const KEY_LEN: usize = 32;

fn argon2_params() -> Params {
    Params::new(65536, 3, 1, Some(KEY_LEN)).expect("argon2 params valid")
}

pub fn derive_key(password: &[u8], salt: &[u8; SALT_LEN]) -> Result<[u8; KEY_LEN]> {
    let mut key = [0u8; KEY_LEN];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params())
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {e}"))?;
    Ok(key)
}

pub fn random_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

fn compress(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::best());
    enc.write_all(data)?;
    Ok(enc.finish()?)
}

fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    let mut dec = ZlibDecoder::new(data);
    let mut out = Vec::new();
    dec.read_to_end(&mut out)?;
    Ok(out)
}

// payload = u32_le(compressed.len()) || compressed || random_padding_to_target
// Both vault volumes share the same target size, so ciphertexts are identical
// in length. Length and padding are inside the AEAD envelope.
fn encrypt_compressed(
    password: &[u8],
    compressed: &[u8],
    target_size: usize,
) -> Result<Encrypted> {
    let mut payload = Vec::with_capacity(4 + target_size);
    payload.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    payload.extend_from_slice(compressed);
    let padding_len = target_size - compressed.len();
    if padding_len > 0 {
        let mut padding = vec![0u8; padding_len];
        OsRng.fill_bytes(&mut padding);
        payload.extend_from_slice(&padding);
    }

    let salt = random_salt();
    let mut raw_key = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&raw_key);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload.as_ref())
        .map_err(|_| anyhow::anyhow!("encryption failed"))?;

    raw_key.zeroize();

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok((salt, out))
}

pub type Encrypted = ([u8; SALT_LEN], Vec<u8>);

/// Encrypt two plaintexts so their ciphertexts are exactly the same length.
/// Use this for all dual-volume sealing — never encrypt volumes independently.
pub fn encrypt_pair(
    decoy_pw: &[u8],
    decoy_pt: &[u8],
    real_pw: &[u8],
    real_pt: &[u8],
) -> Result<(Encrypted, Encrypted)> {
    let decoy_c = compress(decoy_pt)?;
    let real_c = compress(real_pt)?;
    let target = decoy_c.len().max(real_c.len());

    let decoy = encrypt_compressed(decoy_pw, &decoy_c, target)?;
    let real = encrypt_compressed(real_pw, &real_c, target)?;
    Ok((decoy, real))
}

/// Single-volume encrypt (tests and non-vault use). No padding.
#[cfg(test)]
pub fn encrypt(password: &[u8], plaintext: &[u8]) -> Result<Encrypted> {
    let compressed = compress(plaintext)?;
    let target = compressed.len();
    encrypt_compressed(password, &compressed, target)
}

/// Decrypt and decompress. Strips the length prefix written by encrypt_compressed.
pub fn decrypt(password: &[u8], salt: &[u8; SALT_LEN], blob: &[u8]) -> Result<Vec<u8>> {
    if blob.len() < NONCE_LEN {
        bail!("ciphertext too short");
    }
    let (nonce_bytes, ct_with_tag) = blob.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let mut raw_key = derive_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&raw_key);
    let cipher = Aes256Gcm::new(key);

    let payload = cipher
        .decrypt(nonce, ct_with_tag)
        .map_err(|_| anyhow::anyhow!("decryption failed, wrong password or corrupted vault"))?;

    raw_key.zeroize();

    if payload.len() < 4 {
        bail!("decrypted payload too short");
    }
    let compressed_len = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
    if compressed_len > payload.len() - 4 {
        bail!("compressed length field out of range");
    }
    decompress(&payload[4..4 + compressed_len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let plaintext = b"hello nya~ this is secret";
        let (salt, ct) = encrypt(b"password123", plaintext).unwrap();
        let recovered = decrypt(b"password123", &salt, &ct).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn wrong_password_fails() {
        let (salt, ct) = encrypt(b"correct", b"secret").unwrap();
        assert!(decrypt(b"wrong", &salt, &ct).is_err());
    }

    #[test]
    fn different_salts_give_different_ciphertexts() {
        let (salt1, ct1) = encrypt(b"pw", b"data").unwrap();
        let (salt2, ct2) = encrypt(b"pw", b"data").unwrap();
        assert_ne!(salt1, salt2);
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn pair_produces_equal_ct_lengths() {
        let (decoy, real) = encrypt_pair(
            b"decoy_pw", b"short",
            b"real_pw", b"much longer plaintext that compresses differently",
        ).unwrap();
        assert_eq!(decoy.1.len(), real.1.len(), "ciphertext lengths must match for deniability");
    }

    #[test]
    fn pair_roundtrip_both_volumes() {
        let decoy_pt = b"shopping list: eggs, milk";
        let real_pt = b"actual secret: the plans are under the floorboard";
        let ((ds, dct), (rs, rct)) = encrypt_pair(b"dpw", decoy_pt, b"rpw", real_pt).unwrap();
        assert_eq!(decrypt(b"dpw", &ds, &dct).unwrap(), decoy_pt);
        assert_eq!(decrypt(b"rpw", &rs, &rct).unwrap(), real_pt);
    }

    #[test]
    fn compression_roundtrip() {
        let data = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let (salt, ct) = encrypt(b"pw", data).unwrap();
        assert!(ct.len() < data.len() + 200);
        let recovered = decrypt(b"pw", &salt, &ct).unwrap();
        assert_eq!(recovered, data);
    }
}
