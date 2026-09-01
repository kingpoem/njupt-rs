use aes::Aes128;
use cbc::Encryptor;
use cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};

use crate::utils::{Error, Result};

type Aes128CbcEnc = Encryptor<Aes128>;

const WEBVPN_CRYPT_KEY: &[u8; 16] = b"CASB2021EnLink!!";

pub fn encode_host(host: &str) -> Result<String> {
    let mut buf = host.as_bytes().to_vec();
    let msg_len = buf.len();
    buf.resize(msg_len + 16, 0);

    let cipher = Aes128CbcEnc::new_from_slices(WEBVPN_CRYPT_KEY, WEBVPN_CRYPT_KEY)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    let encrypted = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, msg_len)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(hex::encode(encrypted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwglxt_host_hash_is_stable() {
        let hash = encode_host("jwglxt.njupt.edu.cn").unwrap();
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "35f994eca13c99939dc072124de9e7e8d058298921ede2bb096b0d45fab5cbe1"
        );
    }
}
