use rsa::{RsaPublicKey, RsaPrivateKey};
use rsa::pkcs1v15::Pkcs1v15Encrypt;
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Envelope {
    pub encrypted_session_key: Vec<u8>,
    pub encrypted_command: Vec<u8>,
}

impl Envelope {
    pub fn new(encrypted_session_key: Vec<u8>, encrypted_command: Vec<u8>) -> Self {
        Envelope {
            encrypted_session_key,
            encrypted_command,
        }
    }

    pub fn generate_rsa_key_pair() -> (RsaPrivateKey, RsaPublicKey) {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate key pair");
        let public_key = RsaPublicKey::from(&private_key);
        (private_key, public_key)
    }

    pub fn encrypt_session_key(public_key: &RsaPublicKey, session_key: &[u8]) -> Vec<u8> {
        let mut rng = OsRng;
        public_key.encrypt(&mut rng, Pkcs1v15Encrypt, session_key)
            .expect("Failed to encrypt session key")
    }

    pub fn decrypt_session_key(private_key: &RsaPrivateKey, encrypted_session_key: &[u8]) -> Vec<u8> {
        private_key.decrypt(Pkcs1v15Encrypt, encrypted_session_key)
            .expect("Failed to decrypt session key")
    }

    pub fn create_encrypted_envelope(public_key: &RsaPublicKey, command: &[u8], session_key: &[u8]) -> Envelope {
        let encrypted_session_key = Self::encrypt_session_key(public_key, &session_key);
        let encrypted_command = crate::crypto::aes::encrypt(command, &session_key);
        Envelope::new(encrypted_session_key, encrypted_command)
    }

    pub fn decrypt_envelope(private_key: &RsaPrivateKey, envelope: Envelope) -> (Vec<u8>, Vec<u8>) {
        let session_key = Self::decrypt_session_key(private_key, &envelope.encrypted_session_key);
        let decrypted_command = crate::crypto::aes::decrypt(&envelope.encrypted_command, &session_key).unwrap();
        (session_key, decrypted_command)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize Envelope to JSON")
    }
}