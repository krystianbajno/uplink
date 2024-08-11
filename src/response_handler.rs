use crate::command::Response;
use tokio::fs;

pub async fn process_response(response: Response) {
    match response {
        Response::Message { content } => println!("Received message:\n{}\n", content),
        Response::FileList { files } => {
            for file in files {
                println!("{}", file);
            }
            println!("");
        }
        Response::FileData { file_path, data } => {
            println!("{:?} {:?}", file_path, data);
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

        Response::CommandOutput { output } => println!("Command output:\n{}\n", output),
    }
}