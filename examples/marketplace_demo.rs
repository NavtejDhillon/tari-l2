use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tari_l2_common::{Amount, crypto::KeyPair};
use tari_l2_marketplace::MarketplaceManager;
use tari_l2_state_channel::ChannelConfig;
use tari_l2_state_channel::state::Listing;
use tari_l2_marketplace::storage::MarketplaceStorage;

#[tokio::main]
async fn main() {
    println!("=== Tari L2 Marketplace Demo ===\n");

    // Setup
    println!("1. Setting up marketplace...");
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(MarketplaceStorage::open(temp_dir.path()).unwrap());

    let seller_kp = Arc::new(KeyPair::generate());
    let buyer_kp = Arc::new(KeyPair::generate());

    let seller_manager = MarketplaceManager::new(storage.clone(), seller_kp.clone());

    println!("   ✓ Storage initialized");
    println!("   ✓ Seller keypair: {:?}", hex::encode(seller_kp.public_key().as_bytes()));
    println!("   ✓ Buyer keypair: {:?}", hex::encode(buyer_kp.public_key().as_bytes()));

    // Create channel
    println!("\n2. Creating payment channel...");
    let mut balances = HashMap::new();
    balances.insert(seller_kp.public_key(), Amount::new(1000));
    balances.insert(buyer_kp.public_key(), Amount::new(2000));

    let config = ChannelConfig {
        participants: vec![seller_kp.public_key(), buyer_kp.public_key()],
        initial_balances: balances,
        challenge_period: 3600,
    };

    let channel_id = seller_manager.create_channel(config).await.unwrap();
    println!("   ✓ Channel ID: {:?}", hex::encode(channel_id.as_bytes()));

    // Activate channel
    println!("\n3. Activating channel...");
    seller_manager.activate_channel(&channel_id).await.unwrap();
    println!("   ✓ Channel activated");

    // Check initial balances
    println!("\n4. Checking initial balances...");
    let seller_balance = seller_manager.get_balance(&channel_id, &seller_kp.public_key()).await.unwrap();
    let buyer_balance = seller_manager.get_balance(&channel_id, &buyer_kp.public_key()).await.unwrap();
    println!("   ✓ Seller balance: {} units", seller_balance.value());
    println!("   ✓ Buyer balance: {} units", buyer_balance.value());

    // Seller creates a listing
    println!("\n5. Seller creates a product listing...");
    let listing = Listing {
        id: tari_l2_common::crypto::hash_data(b"product_001"),
        seller: seller_kp.public_key(),
        title: "Vintage Laptop".to_string(),
        description: "Dell XPS 13, excellent condition, 16GB RAM".to_string(),
        price: Amount::new(500),
        ipfs_hash: "QmExampleHash123".to_string(),
        active: true,
    };

    let signed_listing = seller_manager.create_listing(&channel_id, listing.clone()).await.unwrap();
    println!("   ✓ Listing created:");
    println!("     - Title: {}", listing.title);
    println!("     - Description: {}", listing.description);
    println!("     - Price: {} units", listing.price.value());
    println!("     - IPFS Hash: {}", listing.ipfs_hash);

    // Get channel info
    println!("\n6. Channel information:");
    let info = seller_manager.get_channel_info(&channel_id).await.unwrap();
    println!("   ✓ Status: {:?}", info.status);
    println!("   ✓ Nonce: {}", info.nonce);
    println!("   ✓ Total collateral: {} units", info.collateral.value());
    println!("   ✓ Participants: {}", info.participants.len());

    // List all channels
    println!("\n7. Listing all channels...");
    let channels = seller_manager.list_channels().await;
    println!("   ✓ Total channels: {}", channels.len());

    // Create a transfer
    println!("\n8. Creating a transfer (state update)...");
    let transfer = seller_manager.transfer(
        &channel_id,
        buyer_kp.public_key(),
        seller_kp.public_key(),
        Amount::new(100)
    ).await.unwrap();
    println!("   ✓ Transfer state update created");
    println!("   ✓ From: Buyer");
    println!("   ✓ To: Seller");
    println!("   ✓ Amount: 100 units");

    println!("\n=== Demo Complete ===");
    println!("\nKey takeaways:");
    println!("• State channels allow off-chain transactions");
    println!("• All state updates require multi-signature verification");
    println!("• Marketplace operations (listings, orders) are state updates");
    println!("• Final state can be committed to L1 for settlement");

    println!("\nNext steps for production:");
    println!("1. Implement P2P message passing for state synchronization");
    println!("2. Add RPC API for external clients");
    println!("3. Connect to actual Tari L1 blockchain");
    println!("4. Implement dispute resolution mechanism");
    println!("5. Add order fulfillment and escrow logic");
}
