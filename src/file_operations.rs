use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn save_file(file_path: &str, data: &[u8]) {
    let mut file = File::create(file_path).await.expect("Failed to create file");
    file.write_all(data).await.expect("Failed to write file");
}
