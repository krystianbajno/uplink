use std::sync::Arc;
use futures_util::stream::SplitSink;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::communication;
use crate::command::Command as NodeCommand;
use crate::envelope::Envelope;
use indoc::indoc;
use crate::shared_state::SharedState;

pub struct TxCommandHandler {
    passphrase: String,
    ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    connection_active: Arc<Mutex<bool>>,
    shared_state: Arc<Mutex<SharedState>>,
}

impl TxCommandHandler {
    pub fn new(
        passphrase: String,
        ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> Self {
        TxCommandHandler { 
            passphrase, 
            ws_sender, 
            connection_active: Arc::new(Mutex::new(true)),
            shared_state,
        }
    }

    pub async fn is_connection_active(&self) -> bool {
        let connection_active = self.connection_active.lock().await;
        *connection_active
    }

    pub async fn handle_command(&mut self, command: &str) {
        let command = command.trim();
    
        if command.is_empty() {
            println!("Empty command received, nothing to execute.");
            return;
        }
    
        {
            let shared_state = self.shared_state.lock().await;
            if shared_state.server_public_key.is_none() || shared_state.session_key.is_none() {
                println!("Session key or public key not available. Initiating handshake...");
                self.send_handshake().await;
                println!("Handshake initiated. Please try the command again after the handshake completes.");
                return;
            }
        }
    
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0).unwrap_or(&"");
        let args = &parts[1..].join(" ");
    
        let node_command = match cmd.to_uppercase().as_str() {
            "H" | "HELP" => {
                let help = indoc!{"
                    [UPLINK HELP]:
    
                    H | HELP - Print help
                    ECHO | PRINT | MSG | TEXT | T - Send a message to connected node.
    
                    GET | DOWNLOAD | D <remote> <local> - Download a file or directory.
                    PUT | UPLOAD | U <local> <remote> - Upload a file or directory.
                    LIST | LS | DIR | L - List files in the directory.
    
                    SHELL | EXEC | RUN | CMD | E | X <command> - Execute a shell command on the connected node.
    
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
            "TEXT" | "ECHO" | "PRINT" | "MSG" | "T" => NodeCommand::Echo { message: args.to_string() },
            "L" | "LIST" | "LS" | "DIR" => NodeCommand::ListFiles,
            "ID" | "WHOAMI" | "WHO" | "W" => NodeCommand::Whoami,
            "PWD" | "WHERE" => NodeCommand::Pwd,
            "USERS"  => NodeCommand::Users,
            "NETSTAT" => NodeCommand::Netstat,
            "N" | "NETWORK" | "IFCONFIG" | "IPCONFIG" => NodeCommand::Network,
            "SYSTEM" | "INFO" | "SYSTEMINFO" | "UNAME" => NodeCommand::Info,
            "D" | "GET" | "DOWNLOAD" => { 
                let arg_parts: Vec<&str> = args.splitn(2, ' ').collect();
    
                if arg_parts.len() < 2 {
                    eprintln!("GET/DOWNLOAD command requires both file path and local path.");
                    return;
                }
    
                let file_path = arg_parts[0].to_string();
                let file_local_path = arg_parts[1].to_string();
    
                NodeCommand::GetFile { file_path, file_local_path }
            },
            "U" | "PUT" | "UPLOAD" => {
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
            "E" | "X" |"SHELL" | "EXEC" | "RUN" | "CMD" => NodeCommand::Execute { command: args.to_string() },
            _ => {
                eprintln!("Unknown command: {}", command);
                return;
            }
        };
    
        let (public_key, session_key) = {
            let shared_state = self.shared_state.lock().await;
            (shared_state.server_public_key.clone(), shared_state.session_key.clone())
        };
    
        if let (Some(public_key), Some(session_key)) = (public_key, session_key) {
            println!("CREATING ENCRYPTED ENVELOPE");

            let envelope = Envelope::create_encrypted_envelope(
                &public_key, 
                &serde_json::to_vec(&node_command).unwrap(), 
                &session_key
            );

            let serialized_envelope = envelope.to_bytes();
            println!("SESSION_KEY - {:?}", session_key);

            println!("ENCRYPTING AND COMPRESSING");

            let encrypted_envelope = communication::prepare_tx(serialized_envelope, &self.passphrase);
    
            if let Some(ws_sender) = &self.ws_sender {
                let mut sender = ws_sender.lock().await;
                println!("SENDING");

                if let Err(e) = communication::send_binary_data(&mut sender, encrypted_envelope).await {
                    eprintln!("Failed to send encrypted envelope: {}", e);
                }
            } else {
                eprintln!("No active WebSocket connection. Command not sent.");
            }
        } else {
            eprintln!("Session key or public key not available. Command not sent.");
        }
    }

    async fn send_handshake(&self) {
        let node_command = NodeCommand::Handshake;

        if let Some(ws_sender) = &self.ws_sender {
            let mut sender = ws_sender.lock().await;
            let serialized_command = serde_json::to_vec(&node_command).expect("Failed to serialize handshake command");
            let encrypted_command = communication::prepare_tx(serialized_command, &self.passphrase);

            if let Err(e) = communication::send_binary_data(&mut sender, encrypted_command).await {
                eprintln!("Failed to send handshake command: {}", e);
            } else {
                println!("Handshake command sent.");
            }
        } else {
            eprintln!("No active WebSocket connection. Handshake not sent.");
        }
    }
}
