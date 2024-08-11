<img src="https://raw.githubusercontent.com/krystianbajno/krystianbajno/main/img/uplink.png"/>

# UPLINK

**UPLINK** is a Rust cross-platform tool for file transfer and remote management that uses AES-256-GCM encryption over WebSockets. It provides robust, real-time communication between clients and servers, allowing for command execution, file transfers, and system management through both command-line and web interface.

## Features

- **Bi-Directional Communication**: Both the server and client can issue commands and receive responses, enabling seamless remote management. Yes, if you connect to the server, the server can execute commands on your computer (you can specify --no-exec flag to disallow that).
- **Secure File Transfers**: Upload and download files or directories with gzip compression and AES-256-GCM encryption, ensuring data integrity and security.
- **Remote Command Execution**: Execute shell commands on the remote server or client, providing powerful control over connected systems.
- **Dynamic Passphrase Management**: Communications are encrypted with AES-256-GCM, with the encryption key derived from a passphrase using HKDF. Passphrases can be updated during runtime, with changes automatically synchronized across all connected nodes.
- **Web Interface**: Manage files, issue commands, and update passphrases directly from a browser-based interface, enhancing accessibility and ease of use.

## Installation

To compile UPLINK, follow these steps:

```bash
git clone <repository_url>
cd uplink
cargo build --release
```

## Command Reference

- **General Commands**
  - `H` - Print help
  - `ECHO | PRINT | MSG` - Send a message to the connected node

- **File Management**
  - `GET | DOWNLOAD <remote> <local>` - Download a file or directory
  - `PUT | UPLOAD <local> <remote>` - Upload a file or directory
  - `LIST | LS | DIR` - List files in the directory

- **Command Execution**
  - `SHELL | EXEC | RUN | CMD <command>` - Execute a shell command on the connected node

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

- **Dynamic WebSocket Connection**: Automatically connects using `ws://` or `wss://` based on the URL.
- **Command Execution**: Issue commands such as `LIST`, `GET`, `PUT`, `SHELL`, etc., directly from the web interface.
- **File Management**: Upload files through the interface; files are compressed and encrypted before being sent to the server.
- **Passphrase Management**: Easily update the passphrase through the web interface. All connected nodes are automatically synchronized with the new passphrase.
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
   Navigate to `http://127.0.0.1:8080` in your browser.
2. **Execute a Command**:  
   Enter `LIST` or `LS` to list files in the server directory.
3. **Upload a File**:  
   Use the file input to select a file and click "Upload" to securely transfer the file to the server.
4. **Change Passphrase**:  
   Update the passphrase by entering a new value in the passphrase field. The update will be synchronized across all connected nodes.

## How It Works

- **Encryption**: All communications are secured using AES-256-GCM encryption to ensure data confidentiality and integrity.
- **Compression**: Data is compressed using gzip before encryption, optimizing transmission speed and reducing bandwidth usage.
- **Passphrase Management**: The encryption passphrase is dynamically managed and can be updated during a session. Updates are automatically synchronized across all connected nodes, ensuring consistent security.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your improvements.

## License

UPLINK is licensed under the WTFPL License. See the [LICENSE](LICENSE) file for more details.