use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{info, error};
use crate::api::{RpcApi, JsonRpcRequest};

/// RPC server
pub struct RpcServer {
    api: Arc<RpcApi>,
    addr: SocketAddr,
}

impl RpcServer {
    pub fn new(api: Arc<RpcApi>, addr: SocketAddr) -> Self {
        Self { api, addr }
    }

    /// Start the RPC server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("RPC server listening on {}", self.addr);

        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    info!("New RPC connection from {}", peer_addr);
                    let api = self.api.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(socket, api).await {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection(
        socket: TcpStream,
        api: Arc<RpcApi>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, mut writer) = socket.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;

            if n == 0 {
                // Connection closed
                break;
            }

            // Parse request
            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    continue;
                }
            };

            // Handle request
            let response = api.handle_request(request).await;

            // Send response
            let response_json = serde_json::to_string(&response)?;
            writer.write_all(response_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }

        Ok(())
    }
}
