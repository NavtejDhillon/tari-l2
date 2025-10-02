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
    l1_connected: Arc<std::sync::atomic::AtomicBool>,
}

impl RpcApi {
    pub fn new(marketplace: Arc<MarketplaceManager>) -> Self {
        Self {
            marketplace,
            l1_connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn new_with_l1(marketplace: Arc<MarketplaceManager>, l1_connected: Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self { marketplace, l1_connected }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        info!("RPC request: {}", request.method);

        let result = match request.method.as_str() {
            "get_node_info" => self.get_node_info().await,
            "get_l1_status" => self.get_l1_status().await,
            "list_channels" => self.list_channels().await,
            "create_channel" => self.create_channel(request.params).await,
            "get_channel_info" => self.get_channel_info(request.params).await,
            "transfer_in_channel" => self.transfer_in_channel(request.params).await,
            "close_channel" => self.close_channel(request.params).await,
            "get_balance" => self.get_balance(request.params).await,
            "create_listing" => self.create_listing(request.params).await,
            "get_listings" => self.get_listings().await,
            "create_order" => self.create_order(request.params).await,
            "get_orders" => self.get_orders().await,
            "update_order_status" => self.update_order_status(request.params).await,
            "transfer" => self.transfer(request.params).await,
            // Escrow operations
            "create_escrow" => self.create_escrow(request.params).await,
            "fund_escrow" => self.fund_escrow(request.params).await,
            "ship_order" => self.ship_order(request.params).await,
            "confirm_delivery" => self.confirm_delivery(request.params).await,
            "request_refund" => self.request_refund(request.params).await,
            "approve_refund" => self.approve_refund(request.params).await,
            "raise_dispute" => self.raise_dispute(request.params).await,
            "get_escrow" => self.get_escrow(request.params).await,
            "list_escrows" => self.list_escrows().await,
            // Wallet operations
            "wallet_create" => self.wallet_create().await,
            "wallet_import_seed" => self.wallet_import_seed(request.params).await,
            "wallet_import_key" => self.wallet_import_key(request.params).await,
            "wallet_export" => self.wallet_export(request.params).await,
            "wallet_sign" => self.wallet_sign(request.params).await,
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

    async fn get_node_info(&self) -> Result<Value, String> {
        // Return basic node information
        Ok(serde_json::json!({
            "public_key": "329e35a4b55ce112e564f72a3d0dde514b7309fa6df45ffd1315e6c921db1bd1",
            "version": "0.1.0",
            "network": "Esmeralda"
        }))
    }

    async fn get_l1_status(&self) -> Result<Value, String> {
        // Return L1 connection status
        let connected = self.l1_connected.load(std::sync::atomic::Ordering::Relaxed);
        Ok(serde_json::json!({
            "connected": connected,
            "network": "Esmeralda",
            "endpoint": "http://127.0.0.1:18142"
        }))
    }

    async fn create_channel(&self, params: Option<Value>) -> Result<Value, String> {
        use tari_l2_state_channel::ChannelConfig;
        use std::collections::HashMap;
        use tari_l2_common::Amount;

        #[derive(serde::Deserialize)]
        struct CreateChannelParams {
            participant1: String,
            participant2: String,
            collateral: u64,
        }

        let params: CreateChannelParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        // Parse public keys from hex
        let pk1_bytes = hex::decode(&params.participant1)
            .map_err(|e| format!("Invalid participant1 hex: {}", e))?;
        let pk1 = PublicKey::from_slice(&pk1_bytes)
            .map_err(|e| format!("Invalid participant1 key: {}", e))?;

        let pk2_bytes = hex::decode(&params.participant2)
            .map_err(|e| format!("Invalid participant2 hex: {}", e))?;
        let pk2 = PublicKey::from_slice(&pk2_bytes)
            .map_err(|e| format!("Invalid participant2 key: {}", e))?;

        // Create channel config
        let mut initial_balances = HashMap::new();
        initial_balances.insert(pk1.clone(), Amount::new(params.collateral / 2));
        initial_balances.insert(pk2.clone(), Amount::new(params.collateral / 2));

        let config = ChannelConfig {
            participants: vec![pk1.clone(), pk2.clone()],
            initial_balances,
            challenge_period: 86400, // 24 hours
        };

        // Create the channel
        let channel_id = self.marketplace.create_channel(config)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "id": hex::encode(channel_id.as_bytes()),
            "status": "created",
            "participant1": params.participant1,
            "participant2": params.participant2,
            "collateral": params.collateral
        }))
    }

    async fn transfer_in_channel(&self, _params: Option<Value>) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "success"
        }))
    }

    async fn close_channel(&self, _params: Option<Value>) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "closed"
        }))
    }

    async fn create_listing(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct CreateListingParams {
            seller_pubkey: String,
            title: String,
            description: String,
            price: u64,
            ipfs_hash: Option<String>,
        }

        let params: CreateListingParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let seller_bytes = hex::decode(&params.seller_pubkey)
            .map_err(|e| format!("Invalid seller_pubkey hex: {}", e))?;
        let seller = PublicKey::from_slice(&seller_bytes)
            .map_err(|e| e.to_string())?;

        // Generate listing ID
        let listing_id = Hash::random();

        // Create global listing (stored in marketplace, not in a specific channel)
        let result = self.marketplace.create_global_listing(
            listing_id,
            seller,
            params.title.clone(),
            params.description.clone(),
            params.price,
            params.ipfs_hash.unwrap_or_else(|| "QmPending".to_string()),
        ).await.map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "id": hex::encode(listing_id.as_bytes()),
            "title": params.title,
            "price": params.price,
            "seller": params.seller_pubkey,
            "status": "active"
        }))
    }

    async fn get_listings(&self) -> Result<Value, String> {
        let listings = self.marketplace.list_all_listings().await;

        let listings_json: Vec<_> = listings.iter().map(|(channel_id, listing)| {
            serde_json::json!({
                "id": hex::encode(listing.id.as_bytes()),
                "channel_id": hex::encode(channel_id.as_bytes()),
                "seller": hex::encode(listing.seller.as_bytes()),
                "title": listing.title,
                "description": listing.description,
                "price": listing.price.value(),
                "ipfs_hash": listing.ipfs_hash,
                "active": listing.active
            })
        }).collect();

        Ok(serde_json::json!(listings_json))
    }

    async fn create_order(&self, params: Option<Value>) -> Result<Value, String> {
        use tari_l2_state_channel::state::{Order, OrderStatus};

        #[derive(serde::Deserialize)]
        struct CreateOrderParams {
            channel_id: String,
            listing_id: String,
            buyer: String,
        }

        let params: CreateOrderParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let channel_id_bytes = hex::decode(&params.channel_id)
            .map_err(|e| format!("Invalid channel_id hex: {}", e))?;
        let channel_id = Hash::from_slice(&channel_id_bytes)
            .map_err(|e| e.to_string())?;

        let listing_id_bytes = hex::decode(&params.listing_id)
            .map_err(|e| format!("Invalid listing_id hex: {}", e))?;
        let listing_id = Hash::from_slice(&listing_id_bytes)
            .map_err(|e| e.to_string())?;

        let buyer_bytes = hex::decode(&params.buyer)
            .map_err(|e| format!("Invalid buyer hex: {}", e))?;
        let buyer = PublicKey::from_slice(&buyer_bytes)
            .map_err(|e| e.to_string())?;

        // Get the listing to find seller and price
        let listings = self.marketplace.get_channel_listings(&channel_id)
            .await
            .map_err(|e| e.to_string())?;

        let listing = listings.iter()
            .find(|l| l.id == listing_id)
            .ok_or("Listing not found")?;

        let order_id = Hash::random();
        let order = Order {
            id: order_id,
            listing_id,
            buyer,
            seller: listing.seller.clone(),
            amount: listing.price,
            status: OrderStatus::Pending,
        };

        // Create signed state update
        let signed_update = self.marketplace.create_order(&channel_id, order.clone())
            .await
            .map_err(|e| e.to_string())?;

        // Apply the update
        self.marketplace.apply_state_update(&channel_id, signed_update)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "id": hex::encode(order_id.as_bytes()),
            "listing_id": params.listing_id,
            "status": "pending"
        }))
    }

    async fn get_orders(&self) -> Result<Value, String> {
        let orders = self.marketplace.list_all_orders().await;

        let orders_json: Vec<_> = orders.iter().map(|(channel_id, order)| {
            serde_json::json!({
                "id": hex::encode(order.id.as_bytes()),
                "channel_id": hex::encode(channel_id.as_bytes()),
                "listing_id": hex::encode(order.listing_id.as_bytes()),
                "buyer": hex::encode(order.buyer.as_bytes()),
                "seller": hex::encode(order.seller.as_bytes()),
                "amount": order.amount.value(),
                "status": format!("{:?}", order.status)
            })
        }).collect();

        Ok(serde_json::json!(orders_json))
    }

    async fn update_order_status(&self, _params: Option<Value>) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "updated"
        }))
    }

    async fn transfer(&self, _params: Option<Value>) -> Result<Value, String> {
        Ok(serde_json::json!({
            "status": "success"
        }))
    }

    // ===== Escrow RPC Methods =====

    async fn create_escrow(&self, params: Option<Value>) -> Result<Value, String> {
        use tari_l2_common::Amount;

        #[derive(serde::Deserialize)]
        struct CreateEscrowParams {
            listing_id: String,
            buyer: String,
            seller: String,
            amount: u64,
            timeout_period: Option<u64>,
        }

        let params: CreateEscrowParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let listing_id_bytes = hex::decode(&params.listing_id)
            .map_err(|e| format!("Invalid listing_id hex: {}", e))?;
        let listing_id = Hash::from_slice(&listing_id_bytes)
            .map_err(|e| e.to_string())?;

        let buyer_bytes = hex::decode(&params.buyer)
            .map_err(|e| format!("Invalid buyer hex: {}", e))?;
        let buyer = PublicKey::from_slice(&buyer_bytes)
            .map_err(|e| e.to_string())?;

        let seller_bytes = hex::decode(&params.seller)
            .map_err(|e| format!("Invalid seller hex: {}", e))?;
        let seller = PublicKey::from_slice(&seller_bytes)
            .map_err(|e| e.to_string())?;

        let escrow_id = self.marketplace.create_escrow(
            listing_id,
            buyer,
            seller,
            Amount::new(params.amount),
            params.timeout_period.unwrap_or(86400), // Default 24 hours
        ).await.map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "id": hex::encode(escrow_id.as_bytes()),
            "status": "created"
        }))
    }

    async fn fund_escrow(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct FundEscrowParams {
            escrow_id: String,
            l1_tx_id: String,
        }

        let params: FundEscrowParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let escrow_id_bytes = hex::decode(&params.escrow_id)
            .map_err(|e| format!("Invalid escrow_id hex: {}", e))?;
        let escrow_id = Hash::from_slice(&escrow_id_bytes)
            .map_err(|e| e.to_string())?;

        self.marketplace.fund_escrow(&escrow_id, params.l1_tx_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "funded"
        }))
    }

    async fn ship_order(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct ShipOrderParams {
            escrow_id: String,
            tracking_info: Option<String>,
        }

        let params: ShipOrderParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let escrow_id_bytes = hex::decode(&params.escrow_id)
            .map_err(|e| format!("Invalid escrow_id hex: {}", e))?;
        let escrow_id = Hash::from_slice(&escrow_id_bytes)
            .map_err(|e| e.to_string())?;

        self.marketplace.ship_order(&escrow_id, params.tracking_info)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "shipped"
        }))
    }

    async fn confirm_delivery(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct ConfirmDeliveryParams {
            escrow_id: String,
        }

        let params: ConfirmDeliveryParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let escrow_id_bytes = hex::decode(&params.escrow_id)
            .map_err(|e| format!("Invalid escrow_id hex: {}", e))?;
        let escrow_id = Hash::from_slice(&escrow_id_bytes)
            .map_err(|e| e.to_string())?;

        self.marketplace.confirm_delivery(&escrow_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "completed"
        }))
    }

    async fn request_refund(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct RequestRefundParams {
            escrow_id: String,
            reason: String,
        }

        let params: RequestRefundParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let escrow_id_bytes = hex::decode(&params.escrow_id)
            .map_err(|e| format!("Invalid escrow_id hex: {}", e))?;
        let escrow_id = Hash::from_slice(&escrow_id_bytes)
            .map_err(|e| e.to_string())?;

        self.marketplace.request_refund(&escrow_id, params.reason)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "refund_requested"
        }))
    }

    async fn approve_refund(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct ApproveRefundParams {
            escrow_id: String,
        }

        let params: ApproveRefundParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let escrow_id_bytes = hex::decode(&params.escrow_id)
            .map_err(|e| format!("Invalid escrow_id hex: {}", e))?;
        let escrow_id = Hash::from_slice(&escrow_id_bytes)
            .map_err(|e| e.to_string())?;

        self.marketplace.approve_refund(&escrow_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "refunded"
        }))
    }

    async fn raise_dispute(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct RaiseDisputeParams {
            escrow_id: String,
            reason: String,
        }

        let params: RaiseDisputeParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let escrow_id_bytes = hex::decode(&params.escrow_id)
            .map_err(|e| format!("Invalid escrow_id hex: {}", e))?;
        let escrow_id = Hash::from_slice(&escrow_id_bytes)
            .map_err(|e| e.to_string())?;

        self.marketplace.raise_dispute(&escrow_id, params.reason)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "disputed"
        }))
    }

    async fn get_escrow(&self, params: Option<Value>) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct GetEscrowParams {
            escrow_id: String,
        }

        let params: GetEscrowParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let escrow_id_bytes = hex::decode(&params.escrow_id)
            .map_err(|e| format!("Invalid escrow_id hex: {}", e))?;
        let escrow_id = Hash::from_slice(&escrow_id_bytes)
            .map_err(|e| e.to_string())?;

        let escrow = self.marketplace.get_escrow(&escrow_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "id": hex::encode(escrow.id.as_bytes()),
            "listing_id": hex::encode(escrow.listing_id.as_bytes()),
            "buyer": hex::encode(escrow.buyer.as_bytes()),
            "seller": hex::encode(escrow.seller.as_bytes()),
            "amount": escrow.amount.value(),
            "status": format!("{:?}", escrow.status),
            "created_at": escrow.created_at.as_secs(),
            "updated_at": escrow.updated_at.as_secs(),
            "timeout_period": escrow.timeout_period,
            "l1_tx_id": escrow.l1_tx_id,
            "tracking_info": escrow.tracking_info,
            "dispute_reason": escrow.dispute_reason
        }))
    }

    async fn list_escrows(&self) -> Result<Value, String> {
        let escrows = self.marketplace.list_escrows().await;

        let escrows_json: Vec<_> = escrows.iter().map(|escrow| {
            serde_json::json!({
                "id": hex::encode(escrow.id.as_bytes()),
                "listing_id": hex::encode(escrow.listing_id.as_bytes()),
                "buyer": hex::encode(escrow.buyer.as_bytes()),
                "seller": hex::encode(escrow.seller.as_bytes()),
                "amount": escrow.amount.value(),
                "status": format!("{:?}", escrow.status),
                "created_at": escrow.created_at.as_secs(),
                "updated_at": escrow.updated_at.as_secs(),
                "timeout_period": escrow.timeout_period,
                "l1_tx_id": &escrow.l1_tx_id,
                "tracking_info": &escrow.tracking_info,
                "dispute_reason": &escrow.dispute_reason
            })
        }).collect();

        Ok(serde_json::json!(escrows_json))
    }

    // ===== Wallet RPC Methods =====

    async fn wallet_create(&self) -> Result<Value, String> {
        use tari_l2_marketplace::Wallet;

        let wallet = Wallet::new();
        let seed_phrase = wallet.generate_seed_phrase();

        Ok(serde_json::json!({
            "address": wallet.address(),
            "public_key": wallet.address(),
            "seed_phrase": seed_phrase,
            "private_key": wallet.export_private_key()
        }))
    }

    async fn wallet_import_seed(&self, params: Option<Value>) -> Result<Value, String> {
        use tari_l2_marketplace::Wallet;

        #[derive(serde::Deserialize)]
        struct ImportSeedParams {
            seed_phrase: String,
        }

        let params: ImportSeedParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let wallet = Wallet::from_seed_phrase(&params.seed_phrase)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "address": wallet.address(),
            "public_key": wallet.address()
        }))
    }

    async fn wallet_import_key(&self, params: Option<Value>) -> Result<Value, String> {
        use tari_l2_marketplace::Wallet;

        #[derive(serde::Deserialize)]
        struct ImportKeyParams {
            private_key: String,
        }

        let params: ImportKeyParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let wallet = Wallet::from_private_key(&params.private_key)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "address": wallet.address(),
            "public_key": wallet.address()
        }))
    }

    async fn wallet_export(&self, params: Option<Value>) -> Result<Value, String> {
        use tari_l2_marketplace::Wallet;

        #[derive(serde::Deserialize)]
        struct ExportParams {
            private_key: String,
        }

        let params: ExportParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let wallet = Wallet::from_private_key(&params.private_key)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "address": wallet.address(),
            "public_key": wallet.address(),
            "private_key": wallet.export_private_key(),
            "seed_phrase": wallet.generate_seed_phrase()
        }))
    }

    async fn wallet_sign(&self, params: Option<Value>) -> Result<Value, String> {
        use tari_l2_marketplace::Wallet;

        #[derive(serde::Deserialize)]
        struct SignParams {
            private_key: String,
            message: String,
        }

        let params: SignParams = serde_json::from_value(
            params.ok_or("Missing parameters")?
        ).map_err(|e| e.to_string())?;

        let wallet = Wallet::from_private_key(&params.private_key)
            .map_err(|e| e.to_string())?;

        let signature = wallet.sign(params.message.as_bytes());

        Ok(serde_json::json!({
            "signature": hex::encode(signature.as_bytes()),
            "public_key": wallet.address()
        }))
    }
}
