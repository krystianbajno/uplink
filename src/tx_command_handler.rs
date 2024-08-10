
use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::communication::{self, prepare_tx};
use crate::compression;
use crate::crypto;

/*
  This is what happens when node sends a command.
*/
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

    
    pub async fn handle_command(&self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0).unwrap_or(&"");
        let args = &parts[1..].join(" ");

        match cmd.to_uppercase().as_str() {
            "ECHO" | "PRINT" | "MSG" => self.print_message(args).await,
            "LIST" | "LS" => self.list_files().await,
            "GET" | "DOWNLOAD" => self.download_file(args).await,
            "PUT" | "UPLOAD" => self.upload_file(args).await,
            "SHELL" | "EXEC" | "RUN" => self.execute_command(args).await,
            "PASSPHRASE" => self.change_passphrase(args).await,
            "PROXY" => self.proxy_to_server(args).await,
            "EXIT" => self.exit_proxy_mode().await,
            _ => eprintln!("Unknown command: {}", command),
        }
    }

    async fn print_message(&self, message: &str) {
        println!("[+] {:?}", message); // todo send message
    }


    pub async fn upload_file(&self, file_path: &str) {
        match fs::read(file_path).await {
            Ok(file_data) => {
                let compressed_data = compression::compress(&file_data);
                let encrypted_data = crypto::encrypt(&compressed_data, self.passphrase.as_bytes());

                if let Some(ws_sender) = &self.ws_sender {
                    communication::send_binary_data(ws_sender, encrypted_data).await;
                }
            }
            Err(e) => eprintln!("Failed to read file {}: {}", file_path, e),
        }
    }

    pub async fn download_file(&self, file_path: &str) {
        let tx = format!("GET {}", file_path).as_bytes();

        if let Some(ws_sender) = &self.ws_sender {
            let encrypted_command = communication::prepare_tx(tx.to_vec(), &self.passphrase.to_string());
            communication::send_binary_data(ws_sender, encrypted_command).await;
        }
    }

    pub async fn execute_command(&self, command: &str) {
        let command_str = format!("SHELL {}", command);

        if let Some(ws_sender) = &self.ws_sender {
            let encrypted_command = crypto::encrypt(command_str.as_bytes(), self.passphrase.as_bytes());
            communication::send_binary_data(ws_sender, encrypted_command).await;
        }
    }

    pub async fn change_passphrase(&self, new_passphrase: &str) {
        self.passphrase = new_passphrase.to_string();
        let tx = format!("PASSPHRASE {}", self.passphrase).as_bytes();
        let response = prepare_tx(tx.to_vec(), &self.passphrase);
        communication::send_binary_data(ws_sender, response).await
    }

    pub async fn handle_responses(&mut self) {
        if let Some(ws_receiver) = &mut self.ws_receiver {
            while let Some(message) = ws_receiver.next().await {
                match message {
                    Ok(Message::Text(response)) => {
                        self.handle_text_response(&response).await;
                    }
                    Ok(Message::Binary(data)) => {
                        self.handle_binary_response(data).await;
                    }
                    _ => eprintln!("Unexpected WebSocket message"),
                }
            }
        }
    }

    async fn handle_text_response(&self, response: &str) {
        // Handle text responses, such as command execution results
        println!("Received response: {}", response);
    }

    async fn handle_binary_response(&self, data: Vec<u8>) {
        // Handle binary responses, such as file data
        let decrypted_data = crypto::decrypt(&data, self.passphrase.as_bytes());
        let decompressed_data = compression::decompress(&decrypted_data);
        
        // Assuming that you know the type of response, handle it accordingly
        // For example, if it's file data, you could save it to disk
        let file_path = "received_file"; // Adjust as needed
        if let Err(e) = fs::write(file_path, decompressed_data).await {
            eprintln!("Failed to write file {}: {}", file_path, e);
        }
    }
}
