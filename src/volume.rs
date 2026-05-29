use anyhow::{bail, Result};
use rand::{RngCore, rngs::OsRng};
use std::fs;
use std::path::Path;

// On-disk vault format (v0.3.0):
//
//   byte 0:                          front_pad_len (u8, random)
//   bytes [1, 1+fpl):                random front padding
//   bytes [1+fpl, 1+fpl+32):         decoy salt
//   bytes [1+fpl+32, 1+fpl+64):      real  salt
//   bytes [1+fpl+64, 1+fpl+64+N):    decoy ciphertext (nonce || ct || tag)
//   bytes [1+fpl+64+N, end-1-bpl):   real  ciphertext (nonce || ct || tag)
//   bytes [end-1-bpl, end-1):        random back padding
//   byte  end-1:                     back_pad_len (u8, random)
//
// Salt positions are at offset (1 + front_pad_len), not fixed.
// CT length requires both boundary bytes to compute.
// Both CTs are always exactly N bytes (guaranteed by encrypt_pair).

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

fn random_bytes(n: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; n];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

impl VaultLayout {
    pub fn new(
        decoy_salt: [u8; SALT_LEN],
        real_salt: [u8; SALT_LEN],
        decoy_ct: Vec<u8>,
        real_ct: Vec<u8>,
    ) -> Self {
        assert_eq!(
            decoy_ct.len(),
            real_ct.len(),
            "ciphertext lengths must match — use encrypt_pair()"
        );
        Self { decoy_salt, real_salt, decoy_ct, real_ct }
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let front_pad_len = OsRng.next_u32() as u8;
        let back_pad_len = OsRng.next_u32() as u8;
        let front_pad = random_bytes(front_pad_len as usize);
        let back_pad = random_bytes(back_pad_len as usize);

        let cap = 2
            + front_pad_len as usize
            + back_pad_len as usize
            + 2 * SALT_LEN
            + self.decoy_ct.len()
            + self.real_ct.len();
        let mut buf = Vec::with_capacity(cap);

        buf.push(front_pad_len);
        buf.extend_from_slice(&front_pad);
        buf.extend_from_slice(&self.decoy_salt);
        buf.extend_from_slice(&self.real_salt);
        buf.extend_from_slice(&self.decoy_ct);
        buf.extend_from_slice(&self.real_ct);
        buf.extend_from_slice(&back_pad);
        buf.push(back_pad_len);

        fs::write(path, &buf)?;
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self> {
        let buf = fs::read(path)?;

        // Need at least: 2 boundary bytes + 2 salts + 2×min CT
        let min_size = 2 + 2 * SALT_LEN + 2 * OVERHEAD;
        if buf.len() < min_size {
            bail!("vault too small to be valid (got {} bytes)", buf.len());
        }

        let front_pad_len = buf[0] as usize;
        let back_pad_len = buf[buf.len() - 1] as usize;

        if buf.len() < min_size + front_pad_len + back_pad_len {
            bail!("vault too small to be valid (got {} bytes)", buf.len());
        }

        let ct_total = buf.len() - 2 - front_pad_len - back_pad_len - 2 * SALT_LEN;
        if !ct_total.is_multiple_of(2) {
            bail!("vault ciphertext region has odd byte count");
        }
        let ct_len = ct_total / 2;

        let salt_start = 1 + front_pad_len;
        let ct_start = salt_start + 2 * SALT_LEN;

        let decoy_salt: [u8; SALT_LEN] =
            buf[salt_start..salt_start + SALT_LEN].try_into().unwrap();
        let real_salt: [u8; SALT_LEN] =
            buf[salt_start + SALT_LEN..ct_start].try_into().unwrap();
        let decoy_ct = buf[ct_start..ct_start + ct_len].to_vec();
        let real_ct = buf[ct_start + ct_len..ct_start + 2 * ct_len].to_vec();

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
