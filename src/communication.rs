use std::sync::Arc;

use futures_util::stream::SplitStream;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{stream::SplitSink, SinkExt};

use tokio::io::{self, AsyncBufReadExt};
use futures_util::stream::StreamExt;

use tokio::sync::Mutex;

use crate::crypto;
use crate::compression;
use crate::rx_command_handler as rxch;
use crate::tx_command_handler as txch;

pub fn prepare_tx(data: Vec<u8>, passphrase: &str) -> Vec<u8> {
    let compressed_data = compression::compress(&data);
    let encrypted_data = crypto::encrypt(&compressed_data, passphrase.as_bytes());
    encrypted_data
}

pub fn prepare_rx(data: Vec<u8>, passphrase: &str) -> Vec<u8> {
    let decrypted_data = crypto::decrypt(&data, passphrase.as_bytes());
    let decompressed_data = compression::decompress(&decrypted_data);
    decompressed_data
}

pub fn prepare_message(message: &str, passphrase: &str) -> Vec<u8> {
    let mut msg = String::from("MSG ");
    msg.push_str(message);
    let data = msg.as_bytes().to_vec();
    
    prepare_tx(data, passphrase)
}

pub async fn send_message(ws_sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, passphrase: &str, message: &str) {
    let msg: Vec<u8> = prepare_message(message, passphrase);
    send_binary_data(ws_sender, msg).await
}

pub async fn send_binary_data(ws_sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: Vec<u8>) {
    ws_sender.send(Message::Binary(data)).await.expect("Failed to send binary data");
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

pub async fn handle_cli(command_handler: Arc<Mutex<txch::TxCommandHandler>>) {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(line) = lines.next_line().await {
        if let Some(command) = line {
            let mut handler = command_handler.lock().await;
            handler.handle_command(&command).await;
        }
    }
}

pub async fn handle_ws_rx_connection(ws_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, command_handler: Arc<Mutex<rxch::RxCommandHandler>>) {
    let mut receiver = ws_receiver;

    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Binary(data)) => {
                let mut handler = command_handler.lock().await;
                handler.handle_responses(data).await;
            }
            _ => eprintln!("Unexpected WebSocket message"),
        }
    }
}
