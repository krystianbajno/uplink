<img src="https://raw.githubusercontent.com/krystianbajno/krystianbajno/main/img/uplink.png"/>


# UPLINK

UPLINK is an AES-GCM encrypted communication tool that leverages **WebSockets** for secure, and bi-directional communication between a client and a server. It supports **command execution**, **file transfers** (upload/download).

## Features

- **Bi-directional Communication:** Both the server and client can issue commands and receive responses.
- **File Transfer:** Upload and download files and directories with gzip compression and AES-256-GCM encryption.
- **Command Execution:** Execute shell commands on the remote server or client.
- **Passphrase Protection:** Communication is encrypted with AES-256-GCM and the key is derived from a passphrase using HKDF, and can be changed during runtime.
- **Web Interface:** A web-based interface allows issuing commands, managing files, and updating the passphrase from a browser.

## Installation

To compile UPLINK:

```sh
git clone <repository_url>
cd uplink
cargo build --release
```

### Command Reference
```bash
H - Print help
ECHO | PRINT | MSG - Send a message to connected node.

GET | DOWNLOAD <remote> <local> - Download a file or directory.
PUT | UPLOAD <local> <remote> - Upload a file or directory.
LIST | LS | DIR - List files in the directory.

SHELL | EXEC | RUN | CMD <command> - Execute a shell command on the connected node.

ID | WHOAMI | WHO | W - Get current user
PWD | WHERE - Get current directory path
USERS - Get users on the system
NETSTAT - Get network connections
N | NETWORK | IFCONFIG | IPCONFIG - Get network adapter configuration
SYSTEM | INFO | SYSTEMINFO | UNAME - Get system configuration

PASSPHRASE - Change the encryption passphrase.
```

## Usage
### Starting the Server
```
./uplink server 127.0.0.1:8080
```
### Starting the Client
```
./uplink client 127.0.0.1:8080
```

### Use precompiled parameters
Modify the parameters in `build.rs` file in order to run default parameters and embed connection instructions inside of binary at compile time.
```
`build.rs`
fn main() {
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_MODE=server");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_ADDRESS=127.0.0.1:8080");
    println!("cargo:rustc-env=CARGO_PKG_METADATA_PRECOMPILED_PASSPHRASE=my_precompiled_passphrase");
}
./uplink
```

### Web Interface
The UPLINK server also hosts a web interface that can be accessed via a browser:

- **Dynamic WebSocket Connection:** Automatically connects using the appropriate `ws://` or `wss://` protocol based on the current URL.
- **Command Execution:** Issue commands like `LIST`, `GET`, `PUT`, `SHELL`, etc., directly from the web interface's input field.
- **File Management:** Upload files directly through the web interface. Uploaded files are compressed and encrypted before being sent to the server.
- **Passphrase Management:** Enter and update the passphrase through the web interface. When the passphrase is updated, the change is propagated to connected nodes, ensuring synchronized encryption.
- **Real-time Feedback:** View command outputs and other feedback directly within the web interface.

### Using the Web Interface

1. **Start the Server:** 
   - Start the UPLINK server using the command:
     ```sh
     ./uplink server 127.0.0.1:8080
     ```
2. **Access the Interface:** 
   - Open a web browser and navigate to `http://<server_ip>:8080`.
3. **Manage Files and Commands:**
   - Use the web interface to upload files, issue commands, and manage the encryption passphrase.
4. **Command Input:**
   - Enter commands in the provided input field and click "Send Command" to execute.
5. **Passphrase Update:**
   - Enter a new passphrase in the designated field. If a previous passphrase existed, the update will automatically be communicated to all connected nodes.

### Example Workflow Using the Web Interface

- **Connect to the Server:**
  - Navigate to `http://127.0.0.1:8080` in your browser after starting the server.
- **Execute a Command:**
  - Enter `LIST` or `LS` to list files in the server directory.
- **Upload a File:**
  - Use the file input to select a file and click "Upload" to securely transfer the file to the server.
- **Change Passphrase:**
  - Update the passphrase by entering a new value in the passphrase field, which will synchronize the passphrase across connected nodes.

### Example Usage

```sh
# Server-side
./uplink server 127.0.0.1:8080

# Client-side
./uplink client 127.0.0.1:8080
SHELL ls

# Using the Web Interface
- Navigate to `http://127.0.0.1:8080` in your browser.
- Enter commands, upload files, or change the passphrase via the web interface.

### How it works
- **Encryption**: All communications are secured using AES-256-GCM encryption.
- **Compression**: Data is compressed using gzip before encryption, reducing transmission size.
- **Passphrase Management**: The passphrase is dynamically managed and can be updated during a session. Updates are automatically synchronized across connected nodes.