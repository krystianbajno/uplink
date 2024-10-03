use crate::enums::response::Response;
use tokio::fs;
use rsa::{pkcs1::DecodeRsaPublicKey, RsaPublicKey};
use crate::shared_state::shared_state::SharedStateHandle;
use crate::crypto::aes::generate_session_key;

pub async fn process_response(response: Response, shared_state: &SharedStateHandle) {
    match response {
        Response::Message { content } => println!("\n{}\n", content),
        Response::FileList { files } => {
            for file in files {
                println!("{}", file);
            }
            println!("");
        }
        Response::FileData { file_path, data } => {
            println!("{:?}", file_path);
            if let Err(e) = fs::write(&file_path, data).await {
                eprintln!("Failed to write file {}: {}", file_path, e);
            }
        }
        Response::UserList {users} => {
            for user in users {
                println!("{}", user);
            }
            println!("");
        }
        Response::Handshake { public_key } => {
            let public_key_pem = String::from_utf8(public_key).expect("Failed to convert public key bytes to string");
            println!("[+] Public key received\n{:?}\n", public_key_pem);

            let mut shared_state = shared_state.lock().await;
            let session_key = generate_session_key();

            if let Ok(decoded_key) = RsaPublicKey::from_pkcs1_pem(&public_key_pem) {
                shared_state.server_public_key = Some(decoded_key);
                shared_state.session_key = Some(session_key.clone());
                println!("[+] Peer public key has been set in shared state.");
                println!("[+] Session key has been set in shared state.");
            } else {
                eprintln!("[!] Failed to decode the received public key.");
            }
        }

        Response::CommandOutput { output } => println!("Command output:\n{}\n", output),
    }
}