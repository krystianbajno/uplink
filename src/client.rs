use tokio::net::TcpStream;
use tokio_tungstenite::client_async;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::rx_command_handler::RxCommandHandler;
use crate::tx_command_handler::TxCommandHandler;
use crate::io::handle_cli;

pub async fn start_client(address: &str, passphrase: Arc<String>, no_exec: bool, no_transfer: bool) {
    let tcp_stream = TcpStream::connect(address).await.expect("Failed to connect to server");
    let url = format!("ws://{}", address);
    let (ws_stream, _) = client_async(&url, tcp_stream)
        .await
        .expect("Failed to upgrade to WebSocket");

    let (ws_sender, ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));
    let ws_receiver = Arc::new(Mutex::new(ws_receiver));

    let tx_command_handler = Arc::new(Mutex::new(TxCommandHandler::new(
        passphrase.clone().to_string(),
        Some(Arc::clone(&ws_sender)),
        Some(Arc::clone(&ws_receiver)),
    )));
    let rx_command_handler = Arc::new(Mutex::new(RxCommandHandler::new(
        passphrase.clone().to_string(),
        Some(Arc::clone(&ws_sender)),
        Some(Arc::clone(&ws_receiver)),
        no_exec,
        no_transfer,
    )));

    tokio::spawn(handle_cli(Arc::clone(&tx_command_handler)));

    tokio::spawn(async move {
        let mut command_handler_for_ws = rx_command_handler.lock().await;
        command_handler_for_ws.handle_rx().await;
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}