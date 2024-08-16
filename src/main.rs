mod uplink_client;
mod uplink_server; 

mod shared_state;
mod crypto;
mod transport;
mod enums;
mod handlers;

use std::sync::Arc;

use tokio::sync::Mutex;
use uplink_server::uplink_server::start_server;
use uplink_client::uplink_client::start_client;
use shared_state::shared_state::SharedState;

#[tokio::main]
async fn main() {
    let (
        mode, 
        address, 
        passphrase, 
        no_exec,
        no_transfer, 
        no_envelope
    ) = get_config();

    let passphrase = Arc::new(passphrase);

    let shared_state = Arc::new(Mutex::new(SharedState::new()));

    match mode.as_deref() {
        Some("server") => {
            let address = address.expect("Address is required for server mode");
            start_server(&address, Arc::clone(&passphrase), no_exec, no_transfer, no_envelope, Arc::clone(&shared_state)).await;
        }
        Some("client") => {
            let address = address.expect("Address is required for client mode");
            start_client(&address, Arc::clone(&passphrase), no_exec, no_transfer, no_envelope, Arc::clone(&shared_state)).await;
        }
        _ => eprintln!("Invalid or missing mode. Use 'server' or 'client'"),
    }
}

fn get_config() -> (Option<String>, Option<String>, String, bool, bool, bool) {
    let precompiled_mode: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_MODE");
    let precompiled_address: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_ADDRESS");
    let precompiled_passphrase: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE");
    let precompiled_no_envelope: Option<bool> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_NO_ENVELOPE")
        .and_then(|s| s.parse::<bool>().ok());

    let args: Vec<String> = std::env::args().collect();

    let mode = args.get(1).cloned().or_else(|| {
        precompiled_mode.map(|s| s.trim_matches('"').to_string())
    });

    let address = args.get(2).cloned().or_else(|| {
        precompiled_address.map(|s| s.trim_matches('"').to_string())
    });

    let no_envelope = args.contains(&"--no-envelope".to_string()) || precompiled_no_envelope.unwrap_or(false);

    let passphrase = std::env::var("PASSPHRASE")
        .or_else(|_| {
            precompiled_passphrase
                .map(|s| s.trim_matches('"').to_string())
                .ok_or(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "default_passphrase".to_string());

    let no_exec = args.contains(&"--no-exec".to_string());
    let no_transfer: bool = args.contains(&"--no-transfer".to_string());    

    (mode, address, passphrase, no_exec, no_transfer, no_envelope)
}
