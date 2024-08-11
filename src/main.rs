mod client;
mod server; 

mod rx_command_handler;
mod tx_command_handler;

mod response_handler;
mod command;
mod communication;
mod crypto;
mod compression;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    let (mode, address, passphrase, no_exec, no_transfer) = get_config();
    let passphrase = Arc::new(passphrase);

    match mode.as_deref() {
        Some("server") => {
            let address = address.expect("Address is required for server mode");
            server::start_server(&address, Arc::clone(&passphrase), no_exec, no_transfer).await;
        }
        Some("client") => {
            let address = address.expect("Address is required for client mode");
            client::start_client(&address, Arc::clone(&passphrase), no_exec, no_transfer).await;
        }
        _ => eprintln!("Invalid or missing mode. Use 'server' or 'client'"),
    }
}
fn get_config() -> (Option<String>, Option<String>, String, bool, bool) {
    let precompiled_mode: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_MODE");
    let precompiled_address: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_ADDRESS");
    let precompiled_passphrase: Option<&str> = option_env!("CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE");

    let args: Vec<String> = std::env::args().collect();

    let mode = args.get(1).cloned().or_else(|| {
        precompiled_mode.map(|s| s.trim_matches('"').to_string())
    });

    let address = args.get(2).cloned().or_else(|| {
        precompiled_address.map(|s| s.trim_matches('"').to_string())
    });

    let passphrase = std::env::var("PASSPHRASE")
        .or_else(|_| {
            precompiled_passphrase
                .map(|s| s.trim_matches('"').to_string())
                .ok_or(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "default_passphrase".to_string());

    let no_exec = args.contains(&"--no-exec".to_string());
    let no_transfer = args.contains(&"--no-transfer".to_string());

    (mode, address, passphrase, no_exec, no_transfer)
}