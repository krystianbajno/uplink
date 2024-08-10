use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use generic_array::GenericArray;
use rand::RngCore;

pub fn derive_key(passphrase: &[u8]) -> Key<Aes256Gcm> {
    let mut key = GenericArray::clone_from_slice(&[0u8; 32]);
    let hkdf = Hkdf::<Sha256>::new(None, passphrase);
    hkdf.expand(&[], &mut key).expect("Key derivation failed");
    Key::from_slice(&key)
}

pub fn generate_nonce() -> Nonce<Aes256Gcm> {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    Nonce::from_slice(&nonce)
}

pub fn encrypt(data: &[u8], passphrase: &[u8]) -> Vec<u8> {
    let key = derive_key(passphrase);
    let cipher = Aes256Gcm::new(&key);
    let nonce = generate_nonce();
    cipher.encrypt(&nonce, data).expect("Encryption failed")
}

pub fn decrypt(encrypted_data: &[u8], passphrase: &[u8]) -> Vec<u8> {
    let key = derive_key(passphrase);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    cipher.decrypt(nonce, &encrypted_data[12..]).expect("Decryption failed")
}
