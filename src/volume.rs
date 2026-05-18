use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

// On-disk vault format (no magic bytes, no headers, no length fields):
//
//   [0..32]         decoy salt
//   [32..64]        real  salt
//   [64..64+N]      decoy ciphertext (nonce || ct || tag)
//   [64+N..64+2N]   real  ciphertext (nonce || ct || tag)
//
// Both ciphertexts are always exactly N bytes. Equal length is guaranteed
// by encrypt_pair(), which pads the shorter plaintext with random bytes
// inside the AES-GCM envelope before encrypting. An observer without a
// password cannot tell the regions apart or prove a second one exists.

pub const SALT_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;
pub const OVERHEAD: usize = NONCE_LEN + TAG_LEN;

#[derive(Debug)]
pub struct VaultLayout {
    pub decoy_salt: [u8; SALT_LEN],
    pub real_salt: [u8; SALT_LEN],
    pub decoy_ct: Vec<u8>,
    pub real_ct: Vec<u8>,
}

impl VaultLayout {
    pub fn new(
        decoy_salt: [u8; SALT_LEN],
        real_salt: [u8; SALT_LEN],
        decoy_ct: Vec<u8>,
        real_ct: Vec<u8>,
    ) -> Self {
        debug_assert_eq!(
            decoy_ct.len(),
            real_ct.len(),
            "ciphertext lengths must match — use encrypt_pair()"
        );
        Self { decoy_salt, real_salt, decoy_ct, real_ct }
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let mut buf =
            Vec::with_capacity(2 * SALT_LEN + self.decoy_ct.len() + self.real_ct.len());
        buf.extend_from_slice(&self.decoy_salt);
        buf.extend_from_slice(&self.real_salt);
        buf.extend_from_slice(&self.decoy_ct);
        buf.extend_from_slice(&self.real_ct);
        fs::write(path, &buf)?;
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self> {
        let buf = fs::read(path)?;
        let min_size = 2 * SALT_LEN + 2 * OVERHEAD;
        if buf.len() < min_size {
            bail!("vault too small to be valid (got {} bytes)", buf.len());
        }

        let ct_total = buf.len() - 2 * SALT_LEN;
        if !ct_total.is_multiple_of(2) {
            bail!("vault ciphertext region has odd byte count");
        }
        let ct_len = ct_total / 2;

        let decoy_salt: [u8; SALT_LEN] = buf[..SALT_LEN].try_into().unwrap();
        let real_salt: [u8; SALT_LEN] = buf[SALT_LEN..2 * SALT_LEN].try_into().unwrap();
        let decoy_ct = buf[2 * SALT_LEN..2 * SALT_LEN + ct_len].to_vec();
        let real_ct = buf[2 * SALT_LEN + ct_len..].to_vec();

        Ok(Self { decoy_salt, real_salt, decoy_ct, real_ct })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn write_read_roundtrip() {
        let decoy_salt = [1u8; SALT_LEN];
        let real_salt = [2u8; SALT_LEN];
        let decoy_ct = vec![0xABu8; 60];
        let real_ct = vec![0xCDu8; 60];

        let path = PathBuf::from("/tmp/test_vault_rr.paw");
        let layout = VaultLayout::new(decoy_salt, real_salt, decoy_ct.clone(), real_ct.clone());
        layout.write(&path).unwrap();

        let loaded = VaultLayout::read(&path).unwrap();
        assert_eq!(loaded.decoy_salt, decoy_salt);
        assert_eq!(loaded.real_salt, real_salt);
        assert_eq!(loaded.decoy_ct, decoy_ct);
        assert_eq!(loaded.real_ct, real_ct);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn equal_length_preserved() {
        let path = PathBuf::from("/tmp/test_vault_eq.paw");
        let layout = VaultLayout::new(
            [3u8; SALT_LEN],
            [4u8; SALT_LEN],
            vec![0xAAu8; 80],
            vec![0xBBu8; 80],
        );
        layout.write(&path).unwrap();

        let loaded = VaultLayout::read(&path).unwrap();
        assert_eq!(loaded.decoy_ct.len(), 80);
        assert_eq!(loaded.real_ct.len(), 80);
        assert!(loaded.decoy_ct.iter().all(|&b| b == 0xAA));
        assert!(loaded.real_ct.iter().all(|&b| b == 0xBB));

        std::fs::remove_file(&path).ok();
    }
}
