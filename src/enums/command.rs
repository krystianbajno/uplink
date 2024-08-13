use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Echo { message: String },
    ListFiles,
    Whoami,
    Info,
    Pwd,
    Users,
    Netstat,
    Network,
    Handshake,
    GetFile { file_path: String, file_local_path: String },
    PutFile { file_path: String, file_up_path: String, data: Vec<u8> },
    Execute { command: String },
}