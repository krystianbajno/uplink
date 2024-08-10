use std::sync::Arc;
use futures_util::stream::SplitSink;
use tokio::io;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::sink::SinkExt;
use tokio::sync::Mutex;
use crate::crypto;
use crate::compression;
use crate::tx_command_handler::TxCommandHandler;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;

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

pub async fn handle_cli(command_handler: Arc<Mutex<TxCommandHandler>>) {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();

    println!("+ CLI Handler UP and running. Enter commands below.");

    while let Ok(Some(command)) = lines.next_line().await {
        let mut handler = command_handler.lock().await;
        handler.handle_command(&command).await;
    }
}

pub async fn is_http_request(stream: &TcpStream) -> bool {
    let mut buffer = [0; 4];
    if let Ok(_) = stream.peek(&mut buffer).await {
        return buffer.starts_with(b"GET ");
    }
    false
}

pub async fn handle_http_request(mut stream: TcpStream) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
        include_str!("static/index.html")
    );
    
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("Failed to write HTTP response: {}", e);
    }

    if let Err(e) = stream.flush().await {
        eprintln!("Failed to flush HTTP response: {}", e);
    }
}
