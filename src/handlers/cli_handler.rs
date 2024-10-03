use tokio::sync::Mutex;
use crate::handlers::tx_command_handler::TxCommandHandler;
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

                let mut handler = command_handler.lock().await;
                if handler.is_connection_active().await {
                    handler.handle_command(command).await;
                } else {
                    println!("Connection inactive. Waiting to reconnect...");
                    break;
                }
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
