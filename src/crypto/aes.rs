use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use rand::RngCore;

use aes_gcm::aead::{Payload, Error as AeadError};
use aes_gcm::aead::generic_array::GenericArray;

pub fn derive_key(passphrase: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(None, passphrase);
    let mut key = [0u8; 32];
    hkdf.expand(&[], &mut key).expect("Key derivation failed");
    key
}

pub fn generate_session_key() -> Vec<u8> {
    let mut session_key = vec![0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut session_key);
    session_key
}

pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn encrypt(data: &[u8], session_key: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new_from_slice(&session_key).expect("Key creation failed");

    let nonce = generate_nonce();
    let mut ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .expect("Encryption failed");

    let mut encrypted_data = nonce.to_vec();
    encrypted_data.append(&mut ciphertext);

    encrypted_data
}

pub fn decrypt(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, AeadError> {
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));

    let (nonce, ciphertext) = encrypted_data.split_at(12);

    cipher.decrypt(Nonce::from_slice(nonce), Payload { msg: ciphertext, aad: &[] })
}
