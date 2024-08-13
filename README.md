# UPLINK
<img src="https://raw.githubusercontent.com/krystianbajno/krystianbajno/main/img/uplink_web.png"/>
<img src="https://raw.githubusercontent.com/krystianbajno/krystianbajno/main/img/uplink.png"/>

**UPLINK** is a Rust cross-platform tool for file transfer and remote management that uses AES-256-GCM encryption over WebSockets. It provides robust, real-time communication between clients and servers, allowing for command execution, file transfers, and system management through both command-line and web interface.

## Crypto description
0. Shared passphrase is used to establish a secure AES-256-GCM channel.
1. Client sends a handshake message to the server.
2. Server responds with public key.
3. Client generates a symmetric session key and encrypts it with public key. Client saves the symmetric session key in state.
4. Client sends an envelope with command to the server. The envelope consists of: { encrypted_symmetric_key, symmetric_key_encrypted_message }.
5. Server receives the envelope and decrypts symmetric session key with private key.
6. Server decrypts the command with symmetric session key.
7. Server executes the command and responds with response encrypted with symmetric_session_key over AES-256-GCM channel.
8. Client receives a response and decrypts it with symmetric_session_key, passphrase, and decompresses the output.

You can disable envelope encryption and use only AES-256-GCM channel using `--no-envelope` switch.

## Features

- **Bi-Directional Communication**: Both the server and client can issue commands and receive responses, enabling seamless remote management. Yes, if you connect to the server, the server can execute commands on your computer (you can specify --no-exec flag to disallow that).
- **Secure File Transfers**: Upload and download files with ease.
- **Remote Command Execution**: Execute shell commands on the remote server or client, providing powerful control over connected systems.
- **Web Interface**: Manage files, issue commands, and update passphrases directly from a browser-based interface, enhancing accessibility and ease of use.
- **Encryption**: All communications are secured using AES-256-GCM encryption Hide from Blue Team with ease.
- **Compression**: Data is compressed using gzip before encryption, optimizing transmission speed and reducing bandwidth usage.

## Installation

To compile UPLINK, follow these steps:

```bash
git clone <repository_url>
cd uplink
cargo build --release
```

## Command Reference

- **General Commands**
  - `HELP | H ` - Print help
  - `TEXT | ECHO | PRINT | MSG | T` - Send a message to the connected node

- **File Management**
  - `GET | D | DOWNLOAD <remote> <local>` - Download a file or directory
  - `PUT | U | UPLOAD <local> <remote>` - Upload a file or directory
  - `LIST | L | LS | DIR` - List files in the directory

- **Command Execution**
  - `E | X | SHELL | EXEC | RUN | CMD <command>` - Execute a shell command on the connected node

- **System Information**
  - `ID | WHOAMI | WHO | W` - Get current user information
  - `PWD | WHERE` - Get the current directory path
  - `USERS` - List users on the system
  - `NETSTAT` - Display network connections
  - `N | NETWORK | IFCONFIG | IPCONFIG` - Get network adapter configuration
  - `SYSTEM | INFO | SYSTEMINFO | UNAME` - Get system configuration details

- **Encryption Management**
  - `PASSPHRASE` - Change the encryption passphrase

## Usage

### Starting the Server

```bash
PASSPHRASE=SetYourStrongPassphraseHere ./uplink server 127.0.0.1:8080
```

### Starting the Client

```bash
PASSPHRASE=SetYourStrongPassphraseHere ./uplink client 127.0.0.1:8080
```

### Disable execution of commands by peers
```bash
./uplink client 127.0.0.1:8000 --no-exec
./uplink server 127.0.0.1:8000 --no-exec
```

### Disallow peers from transferring files.
```bash
./uplink client 127.0.0.1:8000 --no-transfer
./uplink server 127.0.0.1:8000 --no-transfer
```

### Disable envelope encryption and use only AES256GCM
```bash
./uplink client 127.0.0.1:8000 --no-envelope
./uplink server 127.0.0.1:8000 --no-envelope
```

### Using Precompiled Parameters

You can preconfigure UPLINK by modifying the parameters in the `build.rs` file. This allows you to embed default connection instructions directly into the binary at compile time:

```rust
fn main() {
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_MODE=server");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_ADDRESS=127.0.0.1:8080");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE=my_precompiled_passphrase");
}
```

Compile and run:

```bash
./uplink
```

## Web Interface

The UPLINK server includes a web interface accessible via any modern web browser:

- **Dynamic WebSocket Connection**: Automatically connects using `ws://` or `wss://` based on the URL. Enter passphrase and press connect.
- **Command Execution**: Issue commands such as `LIST`, `GET`, `PUT`, `SHELL`, etc., directly from the web interface.
- **File Management**: Upload files through the interface; files are compressed and encrypted before being sent to the server.
- **Real-time Feedback**: View command outputs and system feedback directly within the web interface.

### Accessing the Web Interface

1. **Start the Server**:
   ```bash
   ./uplink server 127.0.0.1:8080
   ```
2. **Open a Web Browser**:
   Navigate to `http://<server_ip>:8080`.
3. **Manage Files and Commands**:
   Use the interface to upload files, issue commands, and manage the encryption passphrase.

### Example Workflow Using the Web Interface

1. **Connect to the Server**:  
   Navigate to `http://127.0.0.1:8080` in your browser. Enter the passphrase and press connect.
2. **Execute a Command**:  
   Enter command into the command input and press "Send Command".
3. **Upload a File**:  
   Use the file input to select a file and click "Upload" to securely transfer the file to the server.
4. **Download a file**:  
   The files in current working directory are listed. Click on them to download them.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your improvements.

## License

UPLINK is licensed under the WTFPL License. See the [LICENSE](LICENSE) file for more details.