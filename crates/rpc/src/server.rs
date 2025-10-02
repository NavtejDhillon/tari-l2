use std::net::SocketAddr;
use std::sync::Arc;
use std::convert::Infallible;
use hyper::{
    Body, Method, Request, Response, Server, StatusCode,
    service::{make_service_fn, service_fn},
};
use tracing::{info, error, debug};
use crate::api::{RpcApi, JsonRpcRequest};

/// RPC server with HTTP support for JSON-RPC
pub struct RpcServer {
    api: Arc<RpcApi>,
    addr: SocketAddr,
}

impl RpcServer {
    pub fn new(api: Arc<RpcApi>, addr: SocketAddr) -> Self {
        Self { api, addr }
    }

    /// Start the HTTP JSON-RPC server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let api = self.api.clone();

        let make_svc = make_service_fn(move |_conn| {
            let api = api.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let api = api.clone();
                    async move { handle_request(req, api).await }
                }))
            }
        });

        let server = Server::bind(&self.addr).serve(make_svc);
        info!("RPC server listening on http://{}", self.addr);

        server.await?;
        Ok(())
    }
}

/// Handle incoming HTTP request
async fn handle_request(
    req: Request<Body>,
    api: Arc<RpcApi>,
) -> Result<Response<Body>, Infallible> {
    // Handle CORS preflight
    if req.method() == Method::OPTIONS {
        return Ok(cors_response(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap()));
    }

    // Only accept POST requests for JSON-RPC
    if req.method() != Method::POST {
        return Ok(cors_response(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::from("Method not allowed. Use POST for JSON-RPC"))
            .unwrap()));
    }

    // Read the request body
    let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Ok(cors_response(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("Failed to read request body: {}", e)))
                .unwrap()));
        }
    };

    // Parse JSON-RPC request
    let rpc_request: JsonRpcRequest = match serde_json::from_slice(&body_bytes) {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to parse JSON-RPC request: {}", e);
            let error_response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32700,
                    "message": "Parse error"
                },
                "id": null
            });
            return Ok(cors_response(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&error_response).unwrap()))
                .unwrap()));
        }
    };

    debug!("Received RPC request: method={}, id={:?}", rpc_request.method, rpc_request.id);

    // Handle the request
    let response = api.handle_request(rpc_request).await;

    // Serialize and send response
    let response_json = match serde_json::to_string(&response) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize response: {}", e);
            let error_response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": "Internal error"
                },
                "id": null
            });
            serde_json::to_string(&error_response).unwrap()
        }
    };

    Ok(cors_response(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(response_json))
        .unwrap()))
}

/// Add CORS headers to response
fn cors_response(mut response: Response<Body>) -> Response<Body> {
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "POST, GET, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type".parse().unwrap());
    response
}
