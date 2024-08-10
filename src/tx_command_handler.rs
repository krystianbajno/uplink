use std::sync::Arc;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::communication;
use crate::command::{Command as NodeCommand, Response};
use crate::compression;

pub struct TxCommandHandler {
    passphrase: String,
    ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    ws_receiver: Option<Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>>,
}

impl TxCommandHandler {
    pub fn new(
        passphrase: String,
        ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
        ws_receiver: Option<Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>>,
    ) -> Self {
        TxCommandHandler { passphrase, ws_sender, ws_receiver }
    }

    pub async fn handle_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0).unwrap_or(&"");
        let args = &parts[1..].join(" ");

        let node_command = match cmd.to_uppercase().as_str() {
            "ECHO" | "PRINT" | "MSG" => NodeCommand::Echo { message: args.to_string() },
            "LIST" | "LS" => NodeCommand::ListFiles,
            "GET" | "DOWNLOAD" => NodeCommand::GetFile { file_path: args.to_string() },
            "PUT" | "UPLOAD" => {
                let file_path = args.to_string();
                let data = fs::read(&file_path).await.unwrap_or_else(|_| vec![]);
                NodeCommand::PutFile { file_path, data }
            }
            "SHELL" | "EXEC" | "RUN" => NodeCommand::Execute { command: args.to_string() },
            "PASSPHRASE" => NodeCommand::ChangePassphrase { new_passphrase: args.to_string() },
            _ => {
                eprintln!("Unknown command: {}", command);
                return;
            }
        };

        self.send_command(node_command).await;
        self.wait_for_response().await;
    }

    async fn send_command(&self, command: NodeCommand) {
        let serialized_command = serde_json::to_vec(&command).expect("Failed to serialize command");
        let encrypted_command = communication::prepare_tx(serialized_command, &self.passphrase);

        if let Some(ws_sender) = &self.ws_sender {
            let mut sender = ws_sender.lock().await;
            communication::send_binary_data(&mut sender, encrypted_command).await;
        }
    }

    async fn wait_for_response(&mut self) {
        if let Some(ws_receiver) = &self.ws_receiver {
            while let Some(message) = ws_receiver.lock().await.next().await {
                match message {
                    Ok(Message::Binary(data)) => {
                        let decrypted_data = communication::prepare_rx(data, &self.passphrase);
                        let response: Response = serde_json::from_slice(&decrypted_data).expect("Failed to deserialize response");
                        self.process_response(response).await;
                        break;
                    }
                    Ok(_) => eprintln!("Received unexpected non-binary message"),
                    Err(e) => {
                        eprintln!("Error receiving WebSocket message: {}", e);
                        break;
                    }
                }
            }
        }
    }

    async fn process_response(&self, response: Response) {
        match response {
            Response::Message { content } => println!("Received message: {}", content),
            Response::FileList { files } => println!("Received file list: {:?}", files),
            Response::FileData { file_path, data } => {
                let decompressed_data = compression::decompress(&data);
                if let Err(e) = fs::write(&file_path, decompressed_data).await {
                    eprintln!("Failed to write file {}: {}", file_path, e);
                }
            }
            Response::CommandOutput { output } => println!("Command output: {}", output),
        }
    }
}
