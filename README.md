# TariMarket L2 - Decentralized Marketplace on Tari

A Layer 2 marketplace solution built on top of Tari blockchain, featuring state channels for instant transactions, P2P listing broadcasts, user profiles with wallet-based identity, and a complete web interface.

## 🎯 Overview

TariMarket L2 is a censorship-resistant, peer-to-peer marketplace that operates as follows:

- **Layer 1 (Tari)**: Handles final settlement, collateral, and dispute resolution
- **Layer 2**: Handles marketplace operations instantly via state channels
- **P2P Network**: Direct node communication with encrypted broadcasts
- **Web Interface**: Full-featured marketplace UI with wallet management
- **Wallet-Based Identity**: Ed25519 keypairs for user authentication
- **Profile System**: User profiles tied to wallet public keys

## 🏗️ Architecture

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
│  │   Tari L1 Client (Collateral & Settlement)   │  │
│  └──────────────────────────────────────────────┘  │
└───────────────────┬─────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────┐
│        Tari Blockchain (Esmeralda Testnet)          │
│  (Collateral Locking, Disputes, Final Settlement)   │
└─────────────────────────────────────────────────────┘
```

## 📁 Project Structure

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
│   │   ├── wallet.rs       # Ed25519 wallet implementation
│   │   └── profile.rs      # User profile system
│   ├── p2p/                # libp2p networking with gossipsub
│   ├── rpc/                # JSON-RPC API (wallet + marketplace)
│   ├── l1-client/          # Tari blockchain integration
│   └── l2-node/            # Main node binary
├── web/                    # Web interface
│   ├── index.html          # Main marketplace UI
│   ├── wallet.html         # Wallet creation/import
│   ├── app.js              # Application logic
│   ├── style.css           # UI styling
│   └── test-data.js        # Test data generators
└── README.md
```

## ✨ Features

### ✅ Implemented

#### Core Infrastructure
- ✅ **State Channels**: Multi-party state channels with balance tracking
- ✅ **P2P Network**: libp2p-based networking with gossipsub
- ✅ **Listing Broadcast**: Censorship-resistant P2P listing distribution
- ✅ **Cryptographic Signatures**: Ed25519 signatures for all operations
- ✅ **Persistent Storage**: Embedded database (sled) for listings & channels
- ✅ **Tari L1 Integration**: Collateral locking on Esmeralda testnet

#### Wallet & Identity
- ✅ **HD Wallets**: BIP39-compatible 12-word seed phrases
- ✅ **Key Management**: Ed25519 keypair generation and storage
- ✅ **Wallet Import/Export**: Seed phrase and private key support
- ✅ **User Profiles**: Name, location, bio, email tied to public key
- ✅ **Profile Editing**: Update profile information anytime

#### Marketplace
- ✅ **Create Listings**: Signed listings broadcast to all peers
- ✅ **Browse Listings**: View all marketplace listings
- ✅ **Place Orders**: Create orders for listings
- ✅ **Order Tracking**: Monitor order status (pending → delivered)
- ✅ **Escrow System**: Automatic escrow for buyer protection
- ✅ **Seller Authentication**: Verify listing ownership via signatures

#### Web Interface
- ✅ **Wallet Setup**: Create or import wallets with seed phrases
- ✅ **Profile Management**: Create and edit user profiles
- ✅ **Marketplace UI**: Browse and create listings
- ✅ **Order Management**: Track all orders
- ✅ **State Channels**: Create and manage payment channels
- ✅ **Debug Console**: Raw RPC interface for testing
- ✅ **Dark/Light Theme**: User preference toggle
- ✅ **Responsive Design**: Works on desktop and mobile

### 🔐 Security Features

- **Signed Listings**: Every listing signed by seller's private key
- **Signature Verification**: Nodes verify signatures before accepting listings
- **Replay Protection**: Timestamp-based replay attack prevention
- **P2P Encryption**: All network messages encrypted
- **No Central Authority**: Fully decentralized P2P architecture
- **Censorship Resistant**: No single node can block listings
- **Persistent Storage**: Listings survive node restarts

## 🚀 Getting Started

### Prerequisites

1. **Rust** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Tari Node** (Esmeralda Testnet)
   - Download from: https://www.tari.com/downloads/
   - Or follow: https://github.com/tari-project/tari

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
   # Download and start Tari testnet node
   # Follow instructions at https://www.tari.com/downloads/
   ```

3. **Configure L2 Node**
   ```bash
   # Edit config.toml (created on first run)
   # Set your Tari node's gRPC address
   ```

4. **Start L2 Node**
   ```bash
   ./target/release/tari-l2-node
   ```

5. **Start Web Interface**
   ```bash
   cd web
   python3 -m http.server 8080
   ```

6. **Open Browser**
   ```
   http://localhost:8080
   ```

## 📖 User Guide

### Creating a Wallet

1. Open http://localhost:8080 in your browser
2. Click "Create New Wallet"
3. **IMPORTANT**: Write down your 12-word seed phrase
4. Click "I have safely backed up my seed phrase"
5. Fill in your profile information (name, location, bio)
6. Click "Complete Setup"

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

## 🧪 Testing Guide

### Running Automated Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p tari-l2-marketplace
cargo test -p tari-l2-state-channel

# With output
cargo test -- --nocapture
```

### Manual Testing Workflow

#### Test 1: Single Node Marketplace

1. **Start Node**
   ```bash
   ./target/release/tari-l2-node
   ```

2. **Create Wallet** (via web UI)
   - Open http://localhost:8080
   - Create new wallet
   - Save seed phrase

3. **Create Listing**
   - Title: "Test Product"
   - Price: 1000000 (1 Tari)
   - Description: "Test description"

4. **Verify Persistence**
   - Restart node
   - Check listing still appears
   - Database stored in `./data/`

#### Test 2: Multi-Node P2P Broadcast

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
   - This proves P2P broadcast works

#### Test 3: Signature Verification

1. **Create Listing**
   - Use web UI to create listing

2. **Check Node Logs**
   ```
   ✅ Created and broadcast global listing: Test Product - 1000000
   📦 Received listing broadcast: Test Product
   ✅ Successfully processed listing from P2P network
   ```

3. **Verify in Database**
   ```bash
   # Listings are stored with signatures
   ls -la ./data/listings/
   ```

#### Test 4: Escrow System

1. **Create Listing as Seller**
2. **Create Order as Buyer**
   - Click "Buy Now" on listing
3. **Check Escrow**
   - Order creates escrow contract
   - Funds locked until delivery confirmed

4. **Complete Order**
   - Seller: Update status → Shipped → Delivered
   - Buyer: Confirm delivery
   - Escrow releases funds

#### Test 5: State Channels

1. **Generate Two Keypairs** (Debug tab → Generate Keypair × 2)
2. **Create Channel**
   - Participant 1: First public key
   - Participant 2: Second public key
   - Collateral: 1000000 µT each

3. **Check L1 Transaction**
   - Node logs show L1 collateral lock
   - View on Tari explorer (testnet)

4. **View Channel Info**
   - Channels list shows balances
   - State updates propagate via P2P

### Test Data Generators

The web UI includes built-in test data generators:

```javascript
// Debug tab → "Test Data Generator"

- Generate Keypair: Creates Ed25519 keypair
- Create 5 Test Channels: Bulk channel creation
- Create 10 Test Listings: Populate marketplace
- Create Test Order: Sample order workflow
```

### RPC Testing (Advanced)

```bash
# Get node info
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_node_info","params":{},"id":1}'

# List listings
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_listings","params":{},"id":1}'

# Create listing
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"create_listing",
    "params":{
      "seller":"<64-char-hex-pubkey>",
      "title":"Test Product",
      "description":"Test",
      "price":1000000,
      "ipfs_hash":"QmTest"
    },
    "id":1
  }'
```

## 📊 Development Roadmap

### ✅ Phase 1: Core Infrastructure (COMPLETED)
- ✅ State channel protocol
- ✅ Marketplace state management
- ✅ P2P networking with libp2p
- ✅ RPC API
- ✅ Storage layer (sled database)
- ✅ Wallet system with Ed25519
- ✅ User profiles
- ✅ Web interface

### ✅ Phase 2: P2P Marketplace (COMPLETED)
- ✅ Signed listing broadcasts
- ✅ Signature verification
- ✅ Listing persistence
- ✅ Multi-node synchronization
- ✅ Censorship-resistant architecture
- ✅ Profile integration with listings
- ✅ Seller authentication

### 🔄 Phase 3: Tari Integration (IN PROGRESS)
- ✅ L1 client implementation
- ✅ Collateral locking on L1
- 🔄 State checkpointing to L1
- 🔄 Dispute resolution mechanism
- 🔄 Challenge period handling
- 🔄 Fraud proofs
- 🔄 Channel force-close on L1

### 📋 Phase 4: Enhanced Features (PLANNED)
- Multi-channel support per participant
- Optimistic state updates
- Watchtower services for monitoring
- Channel rebalancing
- Dynamic fee mechanisms
- Lightning-style routing
- Cross-channel payments

### 🎨 Phase 5: Application Layer (PLANNED)
- IPFS integration for product images
- Dispute resolution UI
- Seller reputation system
- Search and filtering
- Categories and tags
- Messaging between buyers/sellers
- Mobile app (React Native)
- Desktop app (Tauri)

## 🔒 Security Considerations

### Current Implementation

#### Cryptographic Security
- **Ed25519 Signatures**: All state updates and listings signed
- **BLAKE3 Hashing**: Fast cryptographic hashing for state commitments
- **Signature Verification**: All nodes verify signatures independently
- **Replay Protection**: Monotonic nonces and timestamps

#### Network Security
- **P2P Encryption**: All libp2p connections encrypted
- **Peer Authentication**: PeerID derived from public keys
- **No Central Point**: Fully decentralized architecture
- **Censorship Resistance**: No node can block broadcasts

#### Data Security
- **Local Storage**: Wallets stored encrypted in browser localStorage
- **Seed Phrase Export**: Users can backup and restore wallets
- **No Server Storage**: No user data sent to servers

### Known Limitations

1. **Browser Storage**: localStorage can be cleared by user
2. **No Hardware Wallet**: Requires software key management
3. **Testnet Only**: Running on Tari Esmeralda testnet
4. **No Multi-sig**: Single-key wallet only
5. **Limited Dispute Resolution**: Basic escrow only

### Future Security Enhancements

- Hardware wallet support (Ledger, Trezor)
- Multi-signature escrow
- Watchtower services for offline protection
- Fraud proof system
- Time-locked refunds
- Slashing for malicious behavior
- Advanced dispute arbitration

## 🐛 Troubleshooting

### Node Won't Start

```bash
# Check Tari node is running
ps aux | grep tari

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

### Wallet Issues

- **Forgot seed phrase**: Cannot recover, create new wallet
- **Wrong network**: Ensure using Esmeralda testnet
- **Balance issues**: Check Tari block explorer

### Build Errors

```bash
# Missing protoc (common issue)
sudo apt-get install protobuf-compiler  # Debian/Ubuntu
brew install protobuf                    # macOS

# Clear build cache
cargo clean
cargo build --release
```

## 📚 API Reference

See [API.md](docs/API.md) for complete API documentation.

### Key Endpoints

| Method | Description |
|--------|-------------|
| `wallet_create` | Create new HD wallet |
| `wallet_import_seed` | Import from seed phrase |
| `create_listing` | Broadcast new listing |
| `get_listings` | Fetch all listings |
| `create_order` | Place order for listing |
| `get_orders` | Fetch all orders |
| `create_channel` | Create payment channel |
| `list_channels` | List all channels |

## 🤝 Contributing

Contributions welcome! Areas needing development:

1. **L1 Integration**: Complete dispute resolution on Tari
2. **Fraud Proofs**: Implement challenge mechanism
3. **Watchtowers**: Services for offline monitoring
4. **Mobile App**: React Native implementation
5. **Testing**: Comprehensive test coverage
6. **Documentation**: API docs and tutorials
7. **Security Audit**: Professional security review

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

## 📄 License

MIT License - See [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

Built on top of:
- [Tari](https://github.com/tari-project/tari) - Privacy-focused blockchain
- [libp2p](https://libp2p.io/) - Modular P2P networking
- [Ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek) - Digital signatures
- [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) - Cryptographic hashing
- [sled](https://github.com/spacejam/sled) - Embedded database

## 💰 Donations

If you find this project useful, consider supporting development:

- **Monero (XMR)**: `48LDSV3jaHaNWSDoWLg2HPGzKFeSThYg3dzKypiMRN68SsNLGURVhsec6XkjHFFk1K8eUfPEQjwS18tfaRMeVXim33Fca2D`
- **Tari**: `12EU7KJ9ycjuPsmcf9udAaDPoczfMED7tnCVFUgWkFZmsAdaJr5fNDWPzCZp6pzePo19Udi3d7Q8u2ouYMcw46Qq16F`

## 📞 Contact

- **Issues**: https://github.com/yourusername/tari-l2/issues
- **Discussions**: https://github.com/yourusername/tari-l2/discussions

---

**Status**: Active Development | **Version**: 0.2.0-alpha | **Network**: Tari Esmeralda Testnet
