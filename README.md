# TariMarket L2 - Layer 2 Marketplace on Tari

A state channel-based Layer 2 marketplace solution built on top of Tari blockchain, enabling instant, low-cost transactions while maintaining the security and privacy of the Tari network.

## Overview

TariMarket L2 is a Layer 2 scaling solution that operates as follows:

- **Layer 1 (Tari)**: Handles final settlement, disputes, and security
- **Layer 2**: Handles all marketplace operations instantly and cheaply via state channels
- **State Channels**: Enable off-chain transactions that can be settled on-chain
- **P2P Network**: Nodes communicate directly to update channel states
- **RPC API**: Applications connect to L2 nodes for marketplace operations

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Applications                        │
│            (Wallets, Web Apps, CLI)                  │
└───────────────────┬─────────────────────────────────┘
                    │ JSON-RPC
┌───────────────────▼─────────────────────────────────┐
│                L2 Node (TariMarket)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │ Marketplace  │  │ State Channel│  │    P2P    │ │
│  │   Manager    │  │   Protocol   │  │  Network  │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│  ┌──────────────────────────────────────────────┐  │
│  │          Tari L1 Client                      │  │
│  └──────────────────────────────────────────────┘  │
└───────────────────┬─────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────┐
│              Tari Blockchain (L1)                    │
│    (Collateral, Disputes, Final Settlement)          │
└─────────────────────────────────────────────────────┘
```

## Project Structure

```
tari-l2/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── common/             # Common types and utilities
│   ├── state-channel/      # State channel protocol implementation
│   ├── marketplace/        # Marketplace logic and storage
│   ├── p2p/                # P2P networking layer
│   ├── rpc/                # JSON-RPC API
│   └── l2-node/           # Main node binary
└── README.md
```

## Features

### Current Implementation

- ✅ State channel protocol with multi-party support
- ✅ Marketplace state management (listings, orders, balances)
- ✅ P2P networking infrastructure
- ✅ JSON-RPC API for applications
- ✅ Persistent storage with embedded database
- ✅ Cryptographic operations (Ed25519 signatures, BLAKE3 hashing)
- ✅ State update verification and validation
- ✅ Tari L1 integration interface

### Operations Supported

- Create and manage state channels
- Transfer funds between participants
- Create product listings
- Place orders
- Update order status (pending → confirmed → shipping → delivered → completed)
- Query balances and channel state
- Close channels cooperatively

## Building

### Prerequisites

- Rust 1.70 or later
- Tari node running and accessible

### Build from Source

```bash
# Clone the repository
cd tari-l2

# Build the project
cargo build --release

# Run tests
cargo test

# Run the node
cargo run --bin tari-l2-node
```

## Configuration

The node creates a `config.json` on first run with default settings:

```json
{
  "data_dir": "./data",
  "tari_node": {
    "address": "192.168.86.106",
    "port": 18142
  },
  "network": {
    "listen_addr": "/ip4/0.0.0.0/tcp/0",
    "bootstrap_peers": [],
    "max_peers": 50
  },
  "rpc": {
    "listen_addr": "127.0.0.1",
    "port": 18150
  }
}
```

Edit this file to customize your node configuration.

## Usage

### Starting the Node

```bash
cargo run --bin tari-l2-node
```

The node will:
1. Connect to your Tari L1 node at the configured address
2. Load existing channels from storage
3. Start the P2P network
4. Start the RPC server (default: `127.0.0.1:18150`)

### JSON-RPC API

The node exposes a JSON-RPC 2.0 API for applications:

#### List Channels

```bash
echo '{"jsonrpc":"2.0","method":"list_channels","params":null,"id":1}' | \
  nc localhost 18150
```

#### Get Channel Info

```bash
echo '{"jsonrpc":"2.0","method":"get_channel_info","params":{"channel_id":"<hex>"},"id":1}' | \
  nc localhost 18150
```

#### Get Balance

```bash
echo '{"jsonrpc":"2.0","method":"get_balance","params":{"channel_id":"<hex>","participant":"<hex>"},"id":1}' | \
  nc localhost 18150
```

## Development Roadmap

### Phase 1: Core Infrastructure ✅
- State channel protocol
- Marketplace state management
- P2P networking
- RPC API
- Storage layer

### Phase 2: Tari Integration (In Progress)
- Complete Tari L1 client implementation
- Collateral locking/unlocking on L1
- State checkpointing to L1
- Dispute resolution mechanism
- Challenge period handling

### Phase 3: Enhanced Features
- Multi-channel support per participant
- Optimistic state updates
- Watchtower services for monitoring
- Channel rebalancing
- Fee mechanisms

### Phase 4: Application Layer
- Web wallet integration
- Mobile app support
- IPFS integration for product data
- Escrow and dispute resolution UI
- Marketplace discovery

## Security Considerations

### Current Implementation

- Ed25519 signatures for all state updates
- All participants must sign state transitions
- Monotonically increasing nonces prevent replay attacks
- BLAKE3 hashing for state commitments

### Future Enhancements

- Fraud proofs for challenge mechanism
- Watchtower delegation for offline security
- Multi-sig L1 contracts for collateral
- Time-locked challenge periods
- Slashing for malicious behavior

## Testing

Run the full test suite:

```bash
cargo test
```

Run tests for a specific crate:

```bash
cargo test -p tari-l2-state-channel
cargo test -p tari-l2-marketplace
```

## Example: Creating a Channel

```rust
use tari_l2_marketplace::{MarketplaceManager, MarketplaceStorage};
use tari_l2_state_channel::ChannelConfig;
use tari_l2_common::{Amount, crypto::KeyPair};
use std::collections::HashMap;

// Generate keypairs for participants
let alice = KeyPair::generate();
let bob = KeyPair::generate();

// Set initial balances
let mut balances = HashMap::new();
balances.insert(alice.public_key(), Amount::new(1000));
balances.insert(bob.public_key(), Amount::new(1000));

// Create channel config
let config = ChannelConfig {
    participants: vec![alice.public_key(), bob.public_key()],
    initial_balances: balances,
    challenge_period: 3600, // 1 hour
};

// Create marketplace manager
let storage = MarketplaceStorage::open("./data")?;
let manager = MarketplaceManager::new(Arc::new(storage), Arc::new(alice));

// Create channel
let channel_id = manager.create_channel(config).await?;
manager.activate_channel(&channel_id).await?;

println!("Channel created: {}", channel_id);
```

## Contributing

This is an early-stage implementation. Areas needing development:

1. Complete Tari L1 integration
2. Implement fraud proofs and dispute resolution
3. Add watchtower services
4. Build client SDKs
5. Create end-user applications
6. Comprehensive testing and auditing

## License

MIT License

## Acknowledgments

Built on top of:
- [Tari](https://github.com/tari-project/tari) - Privacy-focused blockchain
- [libp2p](https://libp2p.io/) - Modular P2P networking
- [Ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek) - Digital signatures
- [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) - Cryptographic hashing

## Donations

If you find this project useful, consider supporting development:

- **Monero (XMR)**: `12EU7KJ9ycjuPsmcf9udAaDPoczfMED7tnCVFUgWkFZmsAdaJr5fNDWPzCZp6pzePo19Udi3d7Q8u2ouYMcw46Qq16F`
- **Tari**: `12EU7KJ9ycjuPsmcf9udAaDPoczfMED7tnCVFUgWkFZmsAdaJr5fNDWPzCZp6pzePo19Udi3d7Q8u2ouYMcw46Qq16F`

## Contact

For questions and discussions, please open an issue on GitHub.
