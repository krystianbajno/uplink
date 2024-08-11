use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::sink::SinkExt;
use crate::crypto;
use crate::compression;

pub fn prepare_tx(data: Vec<u8>, passphrase: &str) -> Vec<u8> {
    let compressed_data = compression::compress(&data);
    crypto::encrypt(&compressed_data, passphrase.as_bytes())
}

pub fn prepare_rx(data: Vec<u8>, passphrase: &str) -> Vec<u8> {
    let decrypted_data = crypto::decrypt(&data, passphrase.as_bytes());
    compression::decompress(&decrypted_data)
}

pub async fn send_binary_data(ws_sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: Vec<u8>) {
    ws_sender.send(Message::Binary(data)).await.expect("Failed to send binary data");
}

pub async fn is_websocket_upgrade_request(stream: &mut TcpStream) -> bool {
    let mut buffer = [0; 1024];
    if let Ok(n) = stream.peek(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..n]);
        return request.contains("Upgrade: websocket");
    }
    false
}