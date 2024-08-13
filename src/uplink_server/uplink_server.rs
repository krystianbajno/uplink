use tokio::net::TcpListener;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::handlers::cli_handler::handle_cli;
use crate::handlers::rx_command_handler::RxCommandHandler;
use crate::handlers::tx_command_handler::TxCommandHandler;
use crate::shared_state::shared_state::SharedStateHandle;
use tokio_tungstenite::accept_async;
use futures_util::stream::StreamExt;
use crate::transport::communication;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn start_server(
    bind_addr: &str,
    passphrase: Arc<String>,
    no_exec: bool,
    no_transfer: bool,
    no_envelope: bool,
    shared_state: SharedStateHandle,
) {
    let listener = TcpListener::bind(bind_addr).await.unwrap();
    println!("Server listening on {}", bind_addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let shared_state = Arc::clone(&shared_state);

                tokio::spawn(handle_connection(
                    stream,
                    Arc::clone(&passphrase),
                    no_exec,
                    no_transfer,
                    no_envelope,
                    shared_state,
                ));
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {:?}", e);
            }
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    passphrase: Arc<String>,
    no_exec: bool,
    no_transfer: bool,
    no_envelope: bool,
    shared_state: SharedStateHandle,
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
                    no_envelope,
                    Arc::clone(&shared_state), 
                )));
                let rx_command_handler = Arc::new(Mutex::new(RxCommandHandler::new(
                    passphrase.clone().to_string(),
                    Some(Arc::clone(&ws_sender)),
                    Some(Arc::clone(&ws_receiver)),
                    no_exec,
                    no_transfer,
                    no_envelope,
                    Arc::clone(&shared_state),
                )));

                tokio::spawn(handle_cli(Arc::clone(&tx_command_handler)));

                tokio::spawn(async move {
                    let mut command_handler_for_ws = rx_command_handler.lock().await;
                    command_handler_for_ws.handle_rx().await;
                });

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
            Err(e) => {
                eprintln!("WebSocket handshake failed: {:?}", e);
            }
        }
    } else {
        handle_http_request(stream).await;
    }
}

pub async fn handle_http_request(mut stream: TcpStream) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
        include_str!("../static/index.html")
    );
    
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        eprintln!("Failed to write HTTP response: {}", e);
    }

    if let Err(e) = stream.flush().await {
        eprintln!("Failed to flush HTTP response: {}", e);
    }
}
