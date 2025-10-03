# TariMarket L2 - Decentralized Marketplace on Tari

A Layer 2 marketplace solution built on top of Tari blockchain, featuring state channels for instant transactions, P2P listing broadcasts, full Tari wallet integration with UTXO scanning, user profiles with wallet-based identity, and a complete web interface.

## Overview

TariMarket L2 is a censorship-resistant, peer-to-peer marketplace that operates as follows:

- Layer 1 (Tari): Handles final settlement, collateral, and dispute resolution
- Layer 2: Handles marketplace operations instantly via state channels
- P2P Network: Direct node communication with encrypted broadcasts
- Web Interface: Full-featured marketplace UI with wallet management
- Tari Wallet Integration: CipherSeed wallets with 24-word seed phrases and UTXO scanning
- Profile System: User profiles tied to wallet public keys

## Architecture

```
┌─────────────────────────────────────────────────────┐
│             Web Interface (Browser)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
│  │   Wallet     │  │  Marketplace │  │ Profiles │ │
│  │  Management  │  │     UI       │  │   UI     │ │
│  └──────────────┘  └──────────────┘  └──────────┘ │
└───────────────────┬─────────────────────────────────┘
                    │ JSON-RPC (HTTP)
┌───────────────────▼─────────────────────────────────┐
│                L2 Node (TariMarket)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │ Marketplace  │  │ State Channel│  │    P2P    │ │
│  │   Manager    │  │   Protocol   │  │ Broadcast │ │
│  │  + Storage   │  │   + Escrow   │  │  Network  │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│  ┌──────────────────────────────────────────────┐  │
│  │   Tari L1 Client (UTXO Scan & Settlement)    │  │
│  └──────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │   Embedded Tari Wallet (CipherSeed 24-word)  │  │
│  └──────────────────────────────────────────────┘  │
└───────────────────┬─────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────┐
│        Tari Blockchain (Esmeralda Testnet)          │
│  (Mining, Balance Scanning, Collateral, Settlement) │
└─────────────────────────────────────────────────────┘
```

## Project Structure

```
tari-l2/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── common/             # Common types (Hash, PublicKey, Amount)
│   ├── state-channel/      # State channel protocol + marketplace state
│   ├── marketplace/        # Marketplace manager, storage, escrow, profiles
│   │   ├── manager.rs      # P2P broadcast, listing management
│   │   ├── storage.rs      # Persistent database (sled)
│   │   ├── escrow.rs       # Escrow contracts for orders
│   │   ├── wallet.rs       # Tari CipherSeed wallet (24-word seed phrases)
│   │   └── profile.rs      # User profile system
│   ├── p2p/                # libp2p networking with gossipsub
│   ├── rpc/                # JSON-RPC API (wallet + marketplace + balance)
│   ├── l1-client/          # Tari blockchain integration + UTXO scanning
│   └── l2-node/            # Main node binary
├── web/                    # Web interface
│   ├── index.html          # Main marketplace UI
│   ├── wallet.html         # Wallet creation/import
│   ├── app.js              # Application logic
│   ├── style.css           # UI styling
│   └── test-data.js        # Test data generators
├── data/                   # Runtime data (auto-created)
│   ├── current_wallet.json # Active wallet storage
│   └── wallet_*.json       # Wallet backups
└── README.md
```

## Features

### Implemented

#### Core Infrastructure
- State Channels: Multi-party state channels with balance tracking
- P2P Network: libp2p-based networking with gossipsub
- Listing Broadcast: Censorship-resistant P2P listing distribution
- Cryptographic Signatures: Ed25519 signatures for all operations
- Persistent Storage: Embedded database (sled) for listings & channels
- Tari L1 Integration: Collateral locking on Esmeralda testnet

#### Wallet & Identity
- Tari Wallets: CipherSeed-based wallets with 24-word seed phrases (Tari standard)
- Key Management: Ristretto keypair generation using Tari crypto
- Wallet Storage: Automatic wallet persistence to `./data/current_wallet.json`
- Balance Scanning: Real UTXO scanning using wallet's private key
- Output Decryption: Decrypt encrypted outputs to verify ownership
- Wallet Import/Export: Seed phrase and private key support
- User Profiles: Name, location, bio, email tied to public key
- Profile Editing: Update profile information anytime

#### Marketplace
- Create Listings: Signed listings broadcast to all peers
- Browse Listings: View all marketplace listings
- Place Orders: Create orders for listings
- Order Tracking: Monitor order status (pending → delivered)
- Escrow System: Automatic escrow for buyer protection
- Seller Authentication: Verify listing ownership via signatures

#### Web Interface
- Wallet Setup: Create or import wallets with seed phrases
- Balance Display: Real-time balance from blockchain scanning
- Profile Management: Create and edit user profiles
- Marketplace UI: Browse and create listings
- Order Management: Track all orders
- State Channels: Create and manage payment channels
- Debug Console: Raw RPC interface for testing
- Dark/Light Theme: User preference toggle
- Responsive Design: Works on desktop and mobile

### Security Features

- **Tari-Compatible Cryptography**: Uses official Tari crypto libraries
- **CipherSeed Standard**: 24-word seed phrases compatible with Tari console wallet
- **Encrypted Output Scanning**: Decrypt outputs using private view key
- **Signature Verification**: All listings verified with Ed25519 signatures
- **Replay Protection**: Timestamp-based replay attack prevention
- **P2P Encryption**: All network messages encrypted
- **No Central Authority**: Fully decentralized P2P architecture
- **Censorship Resistant**: No single node can block listings
- **Persistent Storage**: Wallets and listings survive node restarts

## Getting Started

### Prerequisites

1. **Rust** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Tari Node** (Esmeralda Testnet)
   - Download from: https://www.tari.com/downloads/
   - Or follow: https://github.com/tari-project/tari
   - **IMPORTANT**: Enable `get_blocks` method in base node config for balance scanning

3. **Web Server** (for web interface)
   ```bash
   # Python 3
   python3 -m http.server 8080

   # Or Node.js
   npx http-server -p 8080
   ```

### Quick Start (5 Minutes)

1. **Clone and Build**
   ```bash
   git clone https://github.com/yourusername/tari-l2.git
   cd tari-l2
   cargo build --release
   ```

2. **Start Tari Node** (in separate terminal)
   ```bash
   # Enable GetBlocks for balance scanning
   # Edit ~/.tari/esmeralda/config/config.toml:
   # Uncomment: "get_blocks" in grpc_server_allow_methods

   minotari_node --network esmeralda
   ```

3. **Start L2 Node**
   ```bash
   ./target/release/tari-l2-node
   ```

4. **Start Web Interface**
   ```bash
   cd web
   python3 -m http.server 8080
   ```

5. **Open Browser**
   ```
   http://localhost:8080
   ```

## User Guide

### Creating a Wallet

1. Open http://localhost:8080 in your browser
2. Click "Create New Wallet"
3. **IMPORTANT**: Write down your 24-word Tari seed phrase
4. The wallet is automatically saved to `./data/current_wallet.json`
5. Your wallet address (emoji format) is displayed

### Checking Your Balance

Balance is automatically scanned from the blockchain:

```bash
# Via RPC (uses saved wallet)
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_l1_balance","params":{"address":"unused"},"id":1}'

# Or with specific seed phrase
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"get_l1_balance",
    "params":{
      "address":"unused",
      "seed_phrase":"your 24 word seed phrase here"
    },
    "id":1
  }'
```

### Creating a Listing

1. Navigate to "Marketplace" tab
2. Fill in the listing form:
   - Title: "Product name"
   - Description: "Product description"
   - Price: Amount in microTari (1 Tari = 1,000,000 µT)
   - Category: Select category
3. Click "Create Listing"
4. Your listing is now broadcast to all peers!

### Viewing Listings

- Go to "Marketplace" tab
- All listings from all nodes appear here
- Your own listings show "(You)" next to seller name
- Click "Buy Now" to create an order

### Managing Orders

1. Go to "Orders" tab
2. View all orders (as buyer or seller)
3. Sellers can update order status:
   - Confirm order
   - Mark as shipped
   - Complete delivery

### Creating State Channels

1. Go to "State Channels" tab
2. Enter two participant public keys (64-char hex)
3. Set collateral amount in µT
4. Click "Create Channel"
5. Channel is created and collateral locked on L1

## Testing Guide

### Running Automated Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p tari-l2-marketplace
cargo test -p tari-l2-state-channel
cargo test -p tari-l2-l1-client

# With output
cargo test -- --nocapture
```

### Manual Testing Workflow

#### Test 1: Wallet Creation and Balance

1. **Create Wallet**
   ```bash
   curl -X POST http://localhost:18000 \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"wallet_create","params":{},"id":1}'
   ```

2. **Verify Wallet Saved**
   ```bash
   cat ./data/current_wallet.json
   # Should show: seed_phrase, private_key, address_hex, etc.
   ```

3. **Check Balance**
   ```bash
   curl -X POST http://localhost:18000 \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"get_l1_balance","params":{"address":"test"},"id":1}'
   # Uses saved wallet automatically
   ```

#### Test 2: UTXO Scanning

1. **Query balance** via RPC
2. **Check node logs** for scanning output
3. **Verify it's real** - Not hardcoded!
   - The scanning uses your wallet's private key
   - Decrypts encrypted outputs
   - Only counts outputs you own

#### Test 3: Multi-Node P2P Broadcast

1. **Start Node 1** (Terminal 1)
   ```bash
   mkdir node1 && cd node1
   ../target/release/tari-l2-node
   # Note the multiaddr from logs
   ```

2. **Start Node 2** (Terminal 2)
   ```bash
   mkdir node2 && cd node2
   # Edit config.toml - add Node 1's multiaddr to bootstrap_peers
   ../target/release/tari-l2-node
   ```

3. **Create Listing on Node 1**
   - Open http://localhost:8080 (Node 1)
   - Create wallet and listing

4. **Verify on Node 2**
   - Open http://localhost:18001 (Node 2 - different port)
   - Listing should appear automatically!

#### Test 4: Wallet Import/Export

1. **Export wallet**
   ```bash
   cat ./data/current_wallet.json
   # Copy the seed_phrase
   ```

2. **Import on another node**
   ```bash
   curl -X POST http://localhost:18000 \
     -H "Content-Type: application/json" \
     -d '{
       "jsonrpc":"2.0",
       "method":"wallet_import_seed",
       "params":{"seed_phrase":"your 24 words here"},
       "id":1
     }'
   ```

3. **Verify same address**
   - Imported wallet should have identical address_hex
   - Balance should be the same (same UTXOs)

### RPC Testing (Advanced)

```bash
# Create wallet
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"wallet_create","params":{},"id":1}'

# Get balance (auto-loads saved wallet)
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_l1_balance","params":{"address":"test"},"id":1}'

# Get balance with specific wallet
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"get_l1_balance",
    "params":{
      "address":"test",
      "seed_phrase":"gate federal tree relief govern..."
    },
    "id":1
  }'

# List listings
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_listings","params":{},"id":1}'
```

## Development Roadmap

### Phase 1: Core Infrastructure (COMPLETED)
- State channel protocol
- Marketplace state management
- P2P networking with libp2p
- RPC API
- Storage layer (sled database)
- User profiles
- Web interface

### Phase 2: P2P Marketplace (COMPLETED)
- Signed listing broadcasts
- Signature verification
- Listing persistence
- Multi-node synchronization
- Censorship-resistant architecture
- Profile integration with listings
- Seller authentication

### Phase 3: Tari Wallet Integration (COMPLETED)
- CipherSeed wallet implementation (24-word Tari standard)
- Wallet persistence to filesystem
- UTXO scanning with private key
- Encrypted output decryption
- Balance checking from blockchain
- Wallet import/export via seed phrases

### Phase 4: L1 Integration (IN PROGRESS)
- L1 client implementation (COMPLETED)
- Collateral locking on L1 (COMPLETED)
- State checkpointing to L1
- Dispute resolution mechanism
- Challenge period handling
- Fraud proofs
- Channel force-close on L1

### Phase 5: Enhanced Features (PLANNED)
- Multi-channel support per participant
- Optimistic state updates
- Watchtower services for monitoring
- Channel rebalancing
- Dynamic fee mechanisms
- Lightning-style routing
- Cross-channel payments

### Phase 6: Application Layer (PLANNED)
- IPFS integration for product images
- Dispute resolution UI
- Seller reputation system
- Search and filtering
- Categories and tags
- Messaging between buyers/sellers
- Mobile app (React Native)
- Desktop app (Tauri)

## Security Considerations

### Current Implementation

#### Cryptographic Security
- **Tari CipherSeed**: Official Tari 24-word seed phrase standard
- **Ristretto Keys**: Same crypto as Tari wallet (RistrettoSecretKey/PublicKey)
- **EncryptedData Decryption**: Uses Tari's encryption scheme for output scanning
- **BLAKE3 Hashing**: Fast cryptographic hashing for state commitments
- **Ed25519 Signatures**: All state updates and listings signed
- **Signature Verification**: All nodes verify signatures independently
- **Replay Protection**: Monotonic nonces and timestamps

#### Wallet Security
- **Persistent Storage**: Wallets saved to `./data/current_wallet.json`
- **Automatic Backup**: Individual wallet files created per address
- **Private Key Protection**: Keys stored locally, never transmitted
- **Seed Phrase Export**: Users can backup and restore wallets

#### Network Security
- **P2P Encryption**: All libp2p connections encrypted
- **Peer Authentication**: PeerID derived from public keys
- **No Central Point**: Fully decentralized architecture
- **Censorship Resistance**: No node can block broadcasts

### Known Limitations

1. **Filesystem Storage**: Wallets stored in plain JSON (encrypt in production)
2. **No Hardware Wallet**: Requires software key management
3. **Testnet Only**: Running on Tari Esmeralda testnet
4. **No Multi-sig**: Single-key wallet only
5. **GetBlocks Required**: Base node must enable `get_blocks` gRPC method

### Future Security Enhancements

- Encrypted wallet storage with passphrase
- Hardware wallet support (Ledger, Trezor)
- Multi-signature escrow
- Watchtower services for offline protection
- Fraud proof system
- Time-locked refunds
- Slashing for malicious behavior
- Advanced dispute arbitration

## Troubleshooting

### Wallet Issues

**Balance shows 0 even after mining**:
1. Check if base node has `get_blocks` enabled:
   ```bash
   # Edit ~/.tari/esmeralda/config/config.toml
   # Under grpc_server_allow_methods, uncomment:
   "get_blocks",
   ```
2. Restart Tari base node
3. Check L2 node logs for scanning errors

**Wallet not saved**:
```bash
# Check if data directory exists
ls -la ./data/

# Manually verify wallet
cat ./data/current_wallet.json
```

**Wrong balance**:
- Ensure you're using the correct seed phrase
- Verify the wallet address matches what was used for mining
- Check Tari block explorer for actual balance

### Node Won't Start

```bash
# Check Tari node is running
ps aux | grep minotari_node

# Check port availability
netstat -an | grep 18000

# Check logs
tail -f tari-l2-node.log
```

### Listings Not Appearing

1. **Check P2P connection**:
   - Look for "Connected to peer" in logs
   - Verify bootstrap_peers in config.toml

2. **Check database**:
   ```bash
   ls -la ./data/listings/
   ```

3. **Try refreshing** browser

### Build Errors

```bash
# Missing protoc (common issue)
sudo apt-get install protobuf-compiler  # Debian/Ubuntu
brew install protobuf                    # macOS

# Clear build cache
cargo clean
cargo build --release
```

## API Reference

### Wallet Endpoints

| Method | Parameters | Description |
|--------|-----------|-------------|
| `wallet_create` | `{}` | Create new Tari wallet with 24-word seed |
| `wallet_import_seed` | `{seed_phrase}` | Import wallet from seed phrase |
| `get_l1_balance` | `{address?, seed_phrase?}` | Get balance (auto-loads saved wallet) |

### Marketplace Endpoints

| Method | Description |
|--------|-------------|
| `create_listing` | Broadcast new listing |
| `get_listings` | Fetch all listings |
| `create_order` | Place order for listing |
| `get_orders` | Fetch all orders |
| `create_channel` | Create payment channel |
| `list_channels` | List all channels |

## Contributing

Contributions welcome! Areas needing development:

1. **Wallet Encryption**: Add passphrase protection for stored wallets
2. **L1 Integration**: Complete dispute resolution on Tari
3. **Fraud Proofs**: Implement challenge mechanism
4. **Watchtowers**: Services for offline monitoring
5. **Mobile App**: React Native implementation
6. **Testing**: Comprehensive test coverage
7. **Documentation**: API docs and tutorials
8. **Security Audit**: Professional security review

### Development Setup

```bash
# Install development tools
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT License - See [LICENSE](LICENSE) for details

## Acknowledgments

Built on top of:
- [Tari](https://github.com/tari-project/tari) - Privacy-focused blockchain
- [libp2p](https://libp2p.io/) - Modular P2P networking
- [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) - Cryptographic hashing
- [sled](https://github.com/spacejam/sled) - Embedded database

## Donations

If you find this project useful, consider supporting development:

- **Monero (XMR)**: `48LDSV3jaHaNWSDoWLg2HPGzKFeSThYg3dzKypiMRN68SsNLGURVhsec6XkjHFFk1K8eUfPEQjwS18tfaRMeVXim33Fca2D`
- **Tari**: `12EU7KJ9ycjuPsmcf9udAaDPoczfMED7tnCVFUgWkFZmsAdaJr5fNDWPzCZp6pzePo19Udi3d7Q8u2ouYMcw46Qq16F`

## Contact

- **Issues**: https://github.com/yourusername/tari-l2/issues
- **Discussions**: https://github.com/yourusername/tari-l2/discussions

---

**Status**: Active Development | **Version**: 0.3.0-alpha | **Network**: Tari Esmeralda Testnet
