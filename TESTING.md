# Testing Guide for Tari L2 Marketplace

## Quick Start

### 1. Run All Tests
```bash
cargo test
```
All 9 unit tests should pass with no warnings.

### 2. Run the Interactive Demo
```bash
cargo run --package tari-l2-marketplace --example marketplace_demo
```

This demonstrates:
- Creating a payment channel between buyer and seller
- Activating the channel
- Checking participant balances
- Creating product listings
- Creating state updates (transfers)
- Viewing channel information

## What's Working

✅ **State Channel Creation** - Channels can be created with multiple participants
✅ **Balance Management** - Participant balances are tracked correctly
✅ **State Updates** - Transfer, listing, and order operations create signed updates
✅ **Persistence** - Channels are stored in Sled database
✅ **Cryptography** - Ed25519 signatures and BLAKE3 hashing
✅ **Multi-signature Framework** - Structure for collecting signatures from all participants

## What Needs Implementation

❌ **Multi-Signature Collection** - Currently only one participant signs
❌ **P2P Networking** - State synchronization between nodes
❌ **RPC API Server** - HTTP/JSON-RPC interface for clients
❌ **L1 Integration** - Actual connection to Tari blockchain
❌ **Dispute Resolution** - Fraud proofs and challenges
❌ **Order Fulfillment** - Complete buyer/seller workflow

## Manual Testing Scenarios

### Test 1: Create Multiple Channels
```bash
# Run the demo multiple times to create different channels
cargo run --package tari-l2-marketplace --example marketplace_demo
```

### Test 2: Unit Tests with Coverage
```bash
# Run tests with output
cargo test -- --nocapture

# Run specific crate tests
cargo test --package tari-l2-common
cargo test --package tari-l2-state-channel
cargo test --package tari-l2-marketplace
```

### Test 3: Build Release Version
```bash
cargo build --release
```

### Test 4: Check for Warnings
```bash
cargo clippy --all-targets --all-features
```

## Architecture Testing Points

### State Channel Layer (`tari-l2-state-channel`)
- ✅ Channel creation and lifecycle
- ✅ State update application
- ✅ Nonce-based ordering
- ⚠️ Multi-signature verification (partial)

### Marketplace Layer (`tari-l2-marketplace`)
- ✅ Marketplace manager operations
- ✅ Storage persistence
- ✅ Listing creation
- ❌ Order lifecycle (needs multi-sig)
- ❌ Payment escrow

### P2P Layer (`tari-l2-p2p`)
- ⚠️ Network structure defined
- ❌ Actual libp2p integration
- ❌ Gossipsub message passing

### RPC Layer (`tari-l2-rpc`)
- ⚠️ JSON-RPC structure defined
- ❌ TCP server not tested
- ❌ Client implementation

### L1 Integration (`tari-l2-node`)
- ⚠️ Client interface defined
- ❌ Tari node connection
- ❌ Collateral locking/unlocking
- ❌ Checkpoint submission

## Performance Testing

Currently not implemented. Future additions:
- Throughput: Transactions per second in a channel
- Latency: State update propagation time
- Scalability: Number of concurrent channels
- Storage: Database growth with channel history

## Integration Testing

To add integration tests, create tests in `tests/` directory:

```rust
// Example: tests/channel_lifecycle.rs
#[tokio::test]
async fn test_full_channel_lifecycle() {
    // Create channel
    // Activate channel
    // Execute transactions
    // Close channel
    // Verify final state
}
```

## Security Testing

⚠️ **Warning**: This is a proof-of-concept. Do NOT use in production without:
- Full security audit
- Penetration testing
- Formal verification of state transition logic
- Comprehensive error handling
- Rate limiting and DOS protection

## Monitoring and Debugging

### Enable Trace Logging
```bash
RUST_LOG=debug cargo run --package tari-l2-marketplace --example marketplace_demo
```

### Check Database Contents
The demo uses temporary directories. To inspect persistent storage:
```rust
// Modify demo to use a fixed path:
let storage = Arc::new(MarketplaceStorage::open("./data").unwrap());
```

Then examine with:
```bash
# Sled database can be read with custom tools or by adding debug prints
```

## Next Steps for Testing

1. **Add Integration Tests** - Full workflow tests
2. **Add Benchmark Tests** - Performance measurements
3. **Add Fuzz Testing** - Random input validation
4. **Add Property Tests** - Invariant checking
5. **Add E2E Tests** - Multiple nodes interacting
6. **Add Load Tests** - Stress testing channels
7. **Add Security Tests** - Attack simulation

## Known Limitations

1. State updates need all signatures but collection logic is incomplete
2. P2P layer is stubbed out
3. No actual L1 blockchain connection
4. No web UI for testing marketplace operations
5. Limited error handling and recovery
6. No metrics or monitoring
