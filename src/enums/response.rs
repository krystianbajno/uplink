use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Message { content: String },
    FileList { files: Vec<String> },
    UserList { users: Vec<String> },
    FileData { file_path: String, data: Vec<u8> },
    CommandOutput { output: String },
    Handshake { public_key: Vec<u8> }
}