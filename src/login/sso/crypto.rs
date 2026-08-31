use aes::Aes128;
use cbc::Encryptor;
use cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};

use crate::utils::{Error, Result};

type Aes128CbcEnc = Encryptor<Aes128>;

// Hardcoded by i.njupt.edu.cn frontend; key/iv bytes are UTF-8 of "iam" + this value.
pub const IAM_CHECK_KEY: &str = "1629428467008";

pub fn iam_encrypt(plaintext: &str, check_key: &str) -> Result<String> {
    let key = format!("iam{check_key}");
    let key = key.as_bytes();
    if key.len() != 16 {
        return Err(Error::Crypto(format!(
            "iam key must be 16 bytes, got {}",
            key.len()
        )));
    }

    let mut buf = plaintext.as_bytes().to_vec();
    let msg_len = buf.len();
    buf.resize(msg_len + 16, 0);

    let cipher = Aes128CbcEnc::new_from_slices(key, key)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    let encrypted = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, msg_len)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(hex::encode(encrypted))
}

pub fn iam_encrypt_default(plaintext: &str) -> Result<String> {
    iam_encrypt(plaintext, IAM_CHECK_KEY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iam_key_is_aes128() {
        assert_eq!(format!("iam{IAM_CHECK_KEY}").len(), 16);
    }

    #[test]
    fn iam_encrypt_is_deterministic() {
        let a = iam_encrypt_default("student").unwrap();
        let b = iam_encrypt_default("student").unwrap();
        assert_eq!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
