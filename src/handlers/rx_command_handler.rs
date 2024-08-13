use std::env;
use std::sync::Arc;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use rsa::pkcs1::EncodeRsaPublicKey;

use tokio::fs;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use users::all_users;
use crate::transport::communication;

use crate::enums::command::Command as NodeCommand;
use crate::enums::response::Response;
use crate::handlers::response_handler::process_response;
use crate::crypto::envelope::Envelope;
use crate::shared_state::shared_state::SharedState;

pub struct RxCommandHandler {
    passphrase: String,
    ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    ws_receiver: Option<Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>>,
    no_exec: bool,
    no_transfer: bool,
    no_envelope: bool,
    shared_state: Arc<Mutex<SharedState>>,
}

impl RxCommandHandler {
    pub fn new(
        passphrase: String,
        ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
        ws_receiver: Option<Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>>,
        no_exec: bool,
        no_transfer: bool,
        no_envelope: bool,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> Self {
        Self {
            passphrase,
            ws_sender,
            ws_receiver,
            no_exec,
            no_transfer,
            no_envelope,
            shared_state,
        }
    }

    pub async fn handle_command(&mut self, command: NodeCommand) -> Response {
        match command {
            NodeCommand::Echo { message } => self.echo_message(&message).await,
            NodeCommand::Info => self.execute_with_permission(|| self.info()).await,
            NodeCommand::Whoami => self.execute_with_permission(|| self.whoami()).await,
            NodeCommand::Pwd => self.execute_with_permission(|| self.pwd()).await,
            NodeCommand::Users => self.execute_with_permission(|| self.users()).await,
            NodeCommand::Netstat => self.execute_with_permission(|| self.netstat()).await,
            NodeCommand::Network => self.execute_with_permission(|| self.network()).await,
            NodeCommand::ListFiles => self.execute_with_transfer_permission(|| self.list_files()).await,
            NodeCommand::GetFile { file_path, file_local_path } => self.execute_with_transfer_permission(|| self.download_file(&file_path, &file_local_path)).await,
            NodeCommand::PutFile { file_path, file_up_path, data } => self.execute_with_transfer_permission(|| self.upload_file(&file_path, &file_up_path, &data)).await,
            NodeCommand::Execute { command } => self.execute_with_permission(|| self.execute_command(&command)).await,
            NodeCommand::Handshake => self.handle_handshake().await,
        }
    }

    async fn execute_with_permission<F, Fut>(&self, func: F) -> Response
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Response>,
    {
        if self.no_exec {
            self.execution_disabled_message()
        } else {
            func().await
        }
    }

    async fn execute_with_transfer_permission<F, Fut>(&self, func: F) -> Response
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Response>,
    {
        if self.no_transfer {
            self.transfer_disabled_message()
        } else {
            func().await
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
        let current_dir = env::current_dir().unwrap();
        Response::Message { content: current_dir.display().to_string() }
    }

    async fn whoami(&self) -> Response {
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .expect("Failed to get the current username");
        Response::Message { content: username }
    }

    async fn users(&self) -> Response {
        let users = unsafe { all_users() };
        let usernames: Vec<String> = users
            .filter_map(|user| user.name().to_str().map(String::from))
            .collect();
        Response::UserList { users: usernames }
    }

    async fn netstat(&self) -> Response {
        Response::Message { content: "NOT IMPLEMENTED".to_string() }
    }

    async fn network(&self) -> Response {
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
            Err(e) => eprintln!("Failed to read directory: {}", e),
        }
        Response::FileList { files: file_list }
    }

    async fn download_file(&self, file_path: &str, file_local_path: &str) -> Response {
        match fs::read(file_path).await {
            Ok(file_data) => Response::FileData { file_path: file_local_path.to_string(), data: file_data },
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
            Command::new("cmd").args(&["/C", command]).output().await
        } else {
            Command::new("sh").arg("-c").arg(command).output().await
        };

        match cmd_result {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout);
                Response::CommandOutput { output: result.to_string() }
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                Response::Message { content: format!("Failed to execute command: {}\n", e) }
            }
        }
    }

    async fn handle_handshake(&mut self) -> Response {
        let (private_key, public_key) = Envelope::generate_rsa_key_pair();
        let mut shared_state = self.shared_state.lock().await;
        shared_state.local_private_key = Some(private_key);
        let public_key_pem = public_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF).expect("Failed to encode public key");
        Response::Handshake { public_key: public_key_pem.as_bytes().to_vec() }
    }

    async fn send_response(&self, response: Response) {
        if let Some(ws_sender) = &self.ws_sender {
            let serialized_response = serde_json::to_vec(&response).expect("Failed to serialize response");
            let encrypted_response = self.encrypt_response(serialized_response).await;
            let mut sender = ws_sender.lock().await;
            if let Err(e) = communication::send_binary_data(&mut sender, encrypted_response).await {
                eprintln!("Failed to send encrypted response: {}", e);
            } else {
                println!("[*] Encrypted response sent.");
            }
        }
    }

    async fn encrypt_response(&self, serialized_response: Vec<u8>) -> Vec<u8> {
        let mut communication_data = communication::prepare_tx(serialized_response, &self.passphrase);
        let session_key = {
            let shared_state = self.shared_state.lock().await;
            shared_state.session_key.clone()
        };

        if let Some(session_key) = session_key {
            communication_data = crate::crypto::aes::encrypt(&communication_data, &session_key);
        }

        communication_data
    }

    async fn decrypt_envelope(&mut self, envelope: Envelope) -> NodeCommand {
        let decrypted_command = {
            let mut shared_state = self.shared_state.lock().await;
            let private_key = shared_state.local_private_key.as_ref().expect("Private key not initialized");
            let (session_key, decrypted_command) = Envelope::decrypt_envelope(private_key, envelope);
            shared_state.session_key = Some(session_key);
            decrypted_command
        };
        serde_json::from_slice(&decrypted_command).expect("Failed to deserialize command")
    }

    pub async fn handle_rx(&mut self) {
        while let Some(message) = self.get_next_message().await {
            println!("\n[+] Received message\n");
            match message {
                Ok(Message::Binary(data)) => {
                    let decrypted_communications = self.decrypt_incoming_message(&data).await;
                    self.process_decrypted_data(decrypted_communications).await;
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

    async fn decrypt_incoming_message(&self, data: &[u8]) -> Vec<u8> {
        let shared_state = self.shared_state.lock().await;
        let decrypted_data = data.to_vec();
        if self.no_envelope {
            communication::prepare_rx(decrypted_data, &self.passphrase)
        } else {
            if let Some(session_key) = &shared_state.session_key {
                if let Ok(decrypted) = crate::crypto::aes::decrypt(&decrypted_data, session_key) {
                    return communication::prepare_rx(decrypted, &self.passphrase);
                }
            }
            communication::prepare_rx(decrypted_data, &self.passphrase)
        }
    }

    async fn process_decrypted_data(&mut self, decrypted_data: Vec<u8>) {
        if let Ok(envelope) = serde_json::from_slice::<Envelope>(&decrypted_data) {
            let command = self.decrypt_envelope(envelope).await;
            let response = self.handle_command(command).await;
            self.send_response(response).await;
        } else if let Ok(command) = serde_json::from_slice::<NodeCommand>(&decrypted_data) {
            if self.no_envelope {
                let response = self.handle_command(command).await;
                self.send_response(response).await;
            } else {
                if let NodeCommand::Handshake = command {
                    let response = self.handle_command(command).await;
                    self.send_response(response).await;
                } else {
                    eprintln!("Received unexpected command during handshake.");
                }
            }
        } else if let Ok(response) = serde_json::from_slice::<Response>(&decrypted_data) {
            process_response(response, &self.shared_state).await;
        } else {
            eprintln!("Received unexpected message format.");
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

    fn execution_disabled_message(&self) -> Response {
        println!("Execution of commands is disabled (--no-exec flag).");
        Response::Message { content: format!("Peer has disabled executing commands.\n") }
    }

    fn transfer_disabled_message(&self) -> Response {
        println!("Transfer is disallowed (--no-transfer flag).");
        Response::Message { content: format!("Transfer is disallowed (--no-transfer flag).\n") }
    }
}
