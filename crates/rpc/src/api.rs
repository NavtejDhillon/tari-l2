use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tari_l2_common::{Hash, PublicKey};
use tari_l2_marketplace::MarketplaceManager;
use tracing::info;

/// JSON-RPC request
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Value,
}

/// JSON-RPC response
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// RPC API implementation
pub struct RpcApi {
    marketplace: Arc<MarketplaceManager>,
}

impl RpcApi {
    pub fn new(marketplace: Arc<MarketplaceManager>) -> Self {
        Self { marketplace }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        info!("RPC request: {}", request.method);

        let result = match request.method.as_str() {
            "list_channels" => self.list_channels().await,
            "get_channel_info" => self.get_channel_info(request.params).await,
            "get_balance" => self.get_balance(request.params).await,
            "create_listing" => self.create_listing(request.params).await,
            "create_order" => self.create_order(request.params).await,
            "update_order_status" => self.update_order_status(request.params).await,
            "transfer" => self.transfer(request.params).await,
            _ => Err(format!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id: request.id,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: e,
                }),
                id: request.id,
            },
        }
    }

    async fn list_channels(&self) -> Result<Value, String> {
        let channels = self.marketplace.list_channels().await;
        serde_json::to_value(channels).map_err(|e| e.to_string())
    }

    async fn get_channel_info(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(Deserialize)]
        struct Params {
            channel_id: String,
        }

        let params: Params = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let channel_id_bytes = hex::decode(&params.channel_id)
            .map_err(|e| e.to_string())?;
        let channel_id = Hash::from_slice(&channel_id_bytes)
            .map_err(|e| e.to_string())?;

        let info = self.marketplace.get_channel_info(&channel_id)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(info).map_err(|e| e.to_string())
    }

    async fn get_balance(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(Deserialize)]
        struct Params {
            channel_id: String,
            participant: String,
        }

        let params: Params = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let channel_id_bytes = hex::decode(&params.channel_id)
            .map_err(|e| e.to_string())?;
        let channel_id = Hash::from_slice(&channel_id_bytes)
            .map_err(|e| e.to_string())?;

        let participant_bytes = hex::decode(&params.participant)
            .map_err(|e| e.to_string())?;
        let participant = PublicKey::from_slice(&participant_bytes)
            .map_err(|e| e.to_string())?;

        let balance = self.marketplace.get_balance(&channel_id, &participant)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(balance.value()).map_err(|e| e.to_string())
    }

    async fn create_listing(&self, _params: Option<Value>) -> Result<Value, String> {
        // Simplified - in real implementation, would parse full listing params
        Ok(Value::String("create_listing not fully implemented".to_string()))
    }

    async fn create_order(&self, _params: Option<Value>) -> Result<Value, String> {
        // Simplified - in real implementation, would parse full order params
        Ok(Value::String("create_order not fully implemented".to_string()))
    }

    async fn update_order_status(&self, _params: Option<Value>) -> Result<Value, String> {
        // Simplified - in real implementation, would parse params
        Ok(Value::String("update_order_status not fully implemented".to_string()))
    }

    async fn transfer(&self, _params: Option<Value>) -> Result<Value, String> {
        // Simplified - in real implementation, would parse params
        Ok(Value::String("transfer not fully implemented".to_string()))
    }
}
