// Copyright (c) 2018-2022 MobileCoin Inc.

//! Cryptographic primitives.

use boring::{
    pkey::Private,
    rsa::{Padding, Rsa},
};

const PKCS1_PADDING_LEN: usize = 11;

/// Encrypt a payload of arbitrary length using a private key.
pub fn encrypt(key: &Rsa<Private>, payload: &[u8]) -> Result<Vec<u8>, String> {
    // Each encrypted chunk must be no longer than the length of the public modulus minus 11 (PKCS1 padding size).
    // (Taken from `rsa::oaep::encrypt`).
    let key_size = key.size() as usize;
    let max_chunk_size = key_size - PKCS1_PADDING_LEN;

    let chunks: Vec<Vec<u8>> = payload
        .chunks(max_chunk_size)
        .map(|chunk| {
            let mut output = vec![0u8; key_size];

            key.private_encrypt(&chunk[..], &mut output, Padding::PKCS1)
                .map_err(|e| format!("encrypt failed: {:?}", e))?;

            Ok(output)
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(chunks
        .into_iter()
        .flat_map(|chunk| chunk.into_iter())
        .collect())
}

/// Decrypt a payload of arbitrary length using a private key.
pub fn decrypt(key: &Rsa<Private>, payload: &[u8]) -> Result<Vec<u8>, String> {
    let key_size = key.size() as usize;

    let chunks: Vec<Vec<u8>> = payload
        .chunks(key_size)
        .map(|chunk| {
            let mut output = vec![0u8; key_size];
            let num_bytes = key
                .private_decrypt(&chunk[..], &mut output, Padding::PKCS1)
                .map_err(|e| format!("decrypt failed: {:?}", e))?;
            output.truncate(num_bytes as usize);
            Ok(output)
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(chunks
        .into_iter()
        .flat_map(|chunk| chunk.into_iter())
        .collect())
}

/// Load a private key from a file
pub fn load_private_key(src: &str) -> Result<Rsa<Private>, String> {
    let key_str = std::fs::read_to_string(src)
        .map_err(|err| format!("failed reading key file {}: {:?}", src, err))?;

    Ok(
        Rsa::private_key_from_pem_passphrase(key_str.as_bytes(), &[])
            .map_err(|err| format!("failed parsing key file {}: {:?}", src, err))?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use boring::pkey::Public;
    use rand_core::{RngCore, SeedableRng};
    use rand_hc::Hc128Rng;

    /// Encrypt a payload of arbitrary length using a public key.
    pub fn encrypt_public(key: &Rsa<Public>, payload: &[u8]) -> Result<Vec<u8>, String> {
        // Each encrypted chunk must be no longer than the length of the public modulus minus 11 (PKCS1 padding size).
        // (Taken from `rsa::oaep::encrypt`).
        let key_size = key.size() as usize;
        let max_chunk_size = key_size - PKCS1_PADDING_LEN;

        let chunks: Vec<Vec<u8>> = payload
            .chunks(max_chunk_size)
            .map(|chunk| {
                let mut output = vec![0u8; key_size];

                key.public_encrypt(&chunk[..], &mut output, Padding::PKCS1)
                    .map_err(|e| format!("encrypt failed: {:?}", e))?;

                Ok(output)
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(chunks
            .into_iter()
            .flat_map(|chunk| chunk.into_iter())
            .collect())
    }

    /// Decrypt a payload of arbitrary length using a public key.
    fn decrypt_public(key: &Rsa<Public>, payload: &[u8]) -> Result<Vec<u8>, String> {
        let key_size = key.size() as usize;

        let chunks: Vec<Vec<u8>> = payload
            .chunks(key_size)
            .map(|chunk| {
                let mut output = vec![0u8; key_size];
                let num_bytes = key
                    .public_decrypt(&chunk[..], &mut output, Padding::PKCS1)
                    .map_err(|e| format!("decrypt failed: {:?}", e))?;
                output.truncate(num_bytes as usize);
                Ok(output)
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(chunks
            .into_iter()
            .flat_map(|chunk| chunk.into_iter())
            .collect())
    }

    #[test]
    fn encrypt_private_decrypt_public_works_with_short_message() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let message = b"this message is less than the key size";

        let encrypted = encrypt(&priv_key, message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize);

        let decrypted = decrypt_public(&pub_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn encrypt_private_decrypt_public_works_with_long_message_that_isnt_a_multiple_of_key_size() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let mut message = vec![0u8; priv_key.size() as usize * 5 + 123]; // message size that does not divide exactly by the key length
        let mut rng = Hc128Rng::from_seed([0u8; 32]);
        rng.fill_bytes(&mut message);

        let encrypted = encrypt(&priv_key, &message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize * 6);

        let decrypted = decrypt_public(&pub_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn encrypt_private_decrypt_public_works_with_long_message_that_is_a_multiple_of_key_size() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let mut message = vec![0u8; priv_key.size() as usize * 4]; // message size that divides exactly by the key length
        let mut rng = Hc128Rng::from_seed([0u8; 32]);
        rng.fill_bytes(&mut message);

        let encrypted = encrypt(&priv_key, &message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize * 5); // longer than message because of padding

        let decrypted = decrypt_public(&pub_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn encrypt_private_decrypt_public_works_with_long_message_that_is_a_multiple_of_chunk_size() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let mut message = vec![0u8; 3 * (priv_key.size() as usize - PKCS1_PADDING_LEN)]; // message size that divides exactly by the chunk size
        let mut rng = Hc128Rng::from_seed([0u8; 32]);
        rng.fill_bytes(&mut message);

        let encrypted = encrypt(&priv_key, &message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize * 3);

        let decrypted = decrypt_public(&pub_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn encrypt_public_decrypt_private_works_with_short_message() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let message = b"this message is less than the key size";

        let encrypted = encrypt_public(&pub_key, message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize);

        let decrypted = decrypt(&priv_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn encrypt_public_decrypt_private_works_with_long_message_that_isnt_a_multiple_of_key_size() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let mut message = vec![0u8; priv_key.size() as usize * 5 + 123]; // message size that does not divide exactly by the key length
        let mut rng = Hc128Rng::from_seed([0u8; 32]);
        rng.fill_bytes(&mut message);

        let encrypted = encrypt_public(&pub_key, &message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize * 6);

        let decrypted = decrypt(&priv_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn encrypt_public_decrypt_private_works_with_long_message_that_is_a_multiple_of_key_size() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let mut message = vec![0u8; priv_key.size() as usize * 4]; // message size that divides exactly by the key length
        let mut rng = Hc128Rng::from_seed([0u8; 32]);
        rng.fill_bytes(&mut message);

        let encrypted = encrypt_public(&pub_key, &message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize * 5); // longer than message because of padding

        let decrypted = decrypt(&priv_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn encrypt_public_decrypt_private_works_with_long_message_that_is_a_multiple_of_chunk_size() {
        let priv_key = Rsa::generate(2048).unwrap();

        let pub_key_pem = priv_key.public_key_to_pem().unwrap();
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).unwrap();

        let mut message = vec![0u8; 3 * (priv_key.size() as usize - PKCS1_PADDING_LEN)]; // message size that divides exactly by the chunk size
        let mut rng = Hc128Rng::from_seed([0u8; 32]);
        rng.fill_bytes(&mut message);

        let encrypted = encrypt_public(&pub_key, &message).unwrap();
        assert_eq!(encrypted.len(), priv_key.size() as usize * 3);

        let decrypted = decrypt(&priv_key, &encrypted).unwrap();
        assert_eq!(message, &decrypted[..]);
    }
}
