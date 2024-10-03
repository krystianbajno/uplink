use std::sync::Arc;
use tokio::sync::Mutex;
use rsa::{RsaPrivateKey, RsaPublicKey};

pub struct SharedState {
    pub server_public_key: Option<RsaPublicKey>,
    pub local_private_key: Option<RsaPrivateKey>,
    pub session_key: Option<Vec<u8>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            local_private_key: None,
            server_public_key: None,
            session_key: None,
        }
    }
}

pub type SharedStateHandle = Arc<Mutex<SharedState>>;