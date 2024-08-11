use tokio::net::TcpListener;
use std::sync::Arc;
use crate::io::handle_client_connection;

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