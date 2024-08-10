use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Echo { message: String },
    ListFiles,
    GetFile { file_path: String },
    PutFile { file_path: String, data: Vec<u8> },
    Execute { command: String },
    ChangePassphrase { new_passphrase: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Message { content: String },
    FileList { files: Vec<String> },
    FileData { file_path: String, data: Vec<u8> },
    CommandOutput { output: String },
}