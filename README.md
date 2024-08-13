# UPLINK
<img src="https://raw.githubusercontent.com/krystianbajno/krystianbajno/main/img/uplink_web.png"/>
<img src="https://raw.githubusercontent.com/krystianbajno/krystianbajno/main/img/uplink.png"/>

**UPLINK** is a Rust cross-platform tool for file transfer and remote management that uses AES-256-GCM and Envelope Encryption over WebSockets. It provides communication between clients and servers, allowing for command execution, file transfers, and system management through both command-line and web interface. Both server and client can issue commands to each other.

You can disable envelope encryption and use only AES-256-GCM channel using `--no-envelope` switch.

You can disallow peers from executing commands using `--no-exec` switch.

You can disallow peers from transferring files using `--no-transfer` switch.

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
  - `HANDSHAKE` - When using envelope encryption, change crypto keys and reestablish a new secure channel.

## Usage

### Starting the Server

```bash
PASSPHRASE=SetYourStrongPassphraseHere ./uplink server 127.0.0.1:8080
```

### Starting the Client

```bash
PASSPHRASE=SetYourStrongPassphraseHere ./uplink client 127.0.0.1:8080
```

### Disallow peers from executing commands
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

The UPLINK server includes a web interface accessible via any modern web browser.

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