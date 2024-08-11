use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::rx_command_handler::RxCommandHandler;
use crate::tx_command_handler::TxCommandHandler;

use crate::communication;

pub async fn start_server(bind_addr: &str, passphrase: Arc<String>, no_exec: bool, no_transfer: bool) {
    let listener = TcpListener::bind(bind_addr).await.unwrap();
    println!("Server listening on {}", bind_addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_client_connection(
                    stream,
                    Arc::clone(&passphrase),
                    no_exec,
                    no_transfer,
                ));
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {:?}", e);
            }
        }
    }
}

async fn handle_client_connection(
    mut stream: TcpStream,
    passphrase: Arc<String>,
    no_exec: bool,
    no_transfer: bool,
) {
    if communication::is_websocket_upgrade_request(&mut stream).await {
        match accept_async(stream).await {
            Ok(ws_stream) => {
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

                tokio::spawn(communication::handle_cli(Arc::clone(&tx_command_handler)));

                tokio::spawn(async move {
                    let mut command_handler_for_ws = rx_command_handler.lock().await;
                    command_handler_for_ws.handle_rx().await;
                });

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                }
            }
            Err(e) => {
                eprintln!("WebSocket handshake failed: {:?}", e);
            }
        }
    } else {
        communication::handle_http_request(stream).await;
    }
}