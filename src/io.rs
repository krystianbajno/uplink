use tokio::sync::Mutex;
use crate::tx_command_handler::TxCommandHandler;
use tokio::io::{self, AsyncBufReadExt};
use std::sync::Arc;

pub async fn handle_cli(command_handler: Arc<Mutex<TxCommandHandler>>) {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();

    print!("[*] UPLINK: CLI Handler is up and running. Enter commands below.\n\n");

    loop {
        match reader.next_line().await {
            Ok(Some(command)) => {
                let command = command.trim();
                if command.is_empty() {
                    continue;
                }

                let handler = command_handler.lock().await;
                handler.handle_command(command).await;
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }
}
