# UPLINK
![UPLINK](https://raw.githubusercontent.com/krystianbajno/krystianbajno/main/img/uplink.png)

**UPLINK** is a Rust-based tool for cross-platform file transfer and remote management. It uses AES-256-GCM encryption over WebSockets. UPLINK supports command execution, file transfers, and system management via command-line and web interfaces. Both server and client can issue commands to each other.

## Security Options

- Disable envelope encryption: `--no-envelope`
- Disable command execution: `--no-exec`
- Disable file transfer: `--no-transfer`
- Disable HTTP server (web interface): `--no-http`

## Installation

To compile UPLINK:

```bash
git clone <repository_url>
cd uplink
cargo build --release
```

## Command Reference

- **General Commands**
  - `HELP | H` - Print help
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
  - `HANDSHAKE` - Change crypto keys and reestablish a new secure channel (envelope encryption only)

## Usage

### Starting the Server

```bash
PASSPHRASE=YourStrongPassphraseHere ./uplink server 127.0.0.1:8080
```

### Starting the Client

```bash
PASSPHRASE=YourStrongPassphraseHere ./uplink client 127.0.0.1:8080
```

### Disable Command Execution

```bash
./uplink client 127.0.0.1:8000 --no-exec
./uplink server 127.0.0.1:8000 --no-exec
```

### Disable File Transfer
```bash
./uplink client 127.0.0.1:8000 --no-transfer
./uplink server 127.0.0.1:8000 --no-transfer
```

### Disable envelope encryption
```bash
./uplink client 127.0.0.1:8000 --no-envelope
./uplink server 127.0.0.1:8000 --no-envelope
```

### Disable HTTP GUI Server
```bash
./uplink server 127.0.0.1:8000 --no-http
```

## Preconfiguring UPLINK
Modify build.rs to embed default settings into the binary:
```rust
fn main() {
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_MODE=server");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_ADDRESS=127.0.0.1:8080");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE=my_precompiled_passphrase");
}
```

Compile and run preconfigured:
```
./uplink
```

## Web Interface
The UPLINK server includes a web interface accessible via any modern web browser. The web interface works only with AES-256-GCM encryption without envelope encryption. Use `--no-envelope` flag.

To use the browser's crypto API, you'll need an SSL session. A simple way to bypass this restriction is by creating a local SSH tunnel:
```bash
ssh -L 8000:<uplink-addr>:<uplink-port> localhost
```

### Accessing the Web Interface

#### 1. Start server
```
./uplink server 127.0.0.1:8080
```

#### 2. Create a tunnel
```
ssh -L 8000:<uplink-addr>:<uplink-port> localhost
```

#### 3. Open a Web Browser:
Go to http://localhost:8000.

### Example Workflow Using the Web Interface

1. **Connect to the Server**:  
   Open `http://localhost:8080` in your web browser. Enter the passphrase and press "Connect."

2. **Execute a Command**:  
   Type the command into the input box and click "Send Command."

3. **Upload a File**:  
   Use the file input to select a file, then click "Upload" to securely transfer the file to the server.

4. **Download a File**:  
   Files in the current directory are listed. Click on a file to download it.

### TODO:
- Add CONNECT and PROXY support
- Support more protocols like QUIC, RTSP, WebRTC
