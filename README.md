# TCP Server

A simple TCP server written in Rust using the Tokio runtime.

## Getting Started

1. Clone the repository:

   ```sh
   git clone https://github.com/ultrasilicon/tcp-server.git
   cd tcp-server
   ```

2. Build and run the server:

   (for blocking impl, change directory into `blocking/`)

   ```sh
   cargo run
   ```

3. Connect to the Server:

   The server listens on port `8888` by default. You can use tools like telnet or netcat to connect:

   ```sh
   nc 127.0.0.1 8888
   ```

