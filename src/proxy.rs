use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn proxy_to_server(proxy_address: &str, mut stream: TcpStream) {
    let mut proxy_stream = TcpStream::connect(proxy_address)
        .await
        .expect("Failed to connect to proxy server");

    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer).await.expect("Failed to read from stream");
        if bytes_read == 0 {
            break;
        }
        proxy_stream.write_all(&buffer[..bytes_read]).await.expect("Failed to write to proxy server");
    }
}
