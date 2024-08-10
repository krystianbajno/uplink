use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use rand::RngCore;

pub fn derive_key(passphrase: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(None, passphrase);
    let mut key = [0u8; 32];
    hkdf.expand(&[], &mut key).expect("Key derivation failed");
    key
}

pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn encrypt(data: &[u8], passphrase: &[u8]) -> Vec<u8> {
    let key = derive_key(passphrase);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Key creation failed");

    let nonce = generate_nonce();
    let mut ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .expect("Encryption failed");

    let mut encrypted_data = nonce.to_vec();
    encrypted_data.append(&mut ciphertext);

    encrypted_data
}

pub fn decrypt(encrypted_data: &[u8], passphrase: &[u8]) -> Vec<u8> {
    let key = derive_key(passphrase);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Key creation failed");

    let (nonce, ciphertext) = encrypted_data.split_at(12);
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .expect("Decryption failed")
}
