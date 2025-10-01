# Tari L2 Marketplace - Testnet Release v0.1.0

## Overview

The Tari L2 Marketplace is a Layer 2 scaling solution for the Tari blockchain, enabling high-throughput, low-latency marketplace transactions through state channels.

## ✨ Features

### Core Functionality
- **State Channels**: Bi-directional payment channels with multi-signature support
- **Marketplace**: Create listings, place orders, and execute trades off-chain
- **P2P Networking**: Decentralized peer-to-peer communication via libp2p
- **Cryptography**: Ed25519 signatures and BLAKE3 hashing
- **Persistence**: Embedded Sled database for state storage
- **RPC API**: JSON-RPC 2.0 interface for external integrations

### Node Features
- **CLI Interface**: Full command-line management
- **Configuration**: TOML/JSON configuration files
- **Logging**: Structured logging with multiple levels
- **Multi-node Support**: Run multiple nodes locally or distributed

## 🚀 Quick Start

```bash
# Build
cargo build --release

# Deploy 3-node testnet
./scripts/deploy_testnet.sh 3

# View logs
tail -f testnet_data/node1/node.log

# Stop testnet
./scripts/stop_testnet.sh
```

See [QUICK_START.md](QUICK_START.md) for detailed instructions.

## 📚 Documentation

- **[QUICK_START.md](QUICK_START.md)** - Get running in 5 minutes
- **[TESTNET_GUIDE.md](TESTNET_GUIDE.md)** - Complete testnet documentation
- **[TESTING.md](TESTING.md)** - Testing guide and procedures
- **[README.md](README.md)** - Technical architecture and design

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     L2 Node                               │
│                                                           │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │     CLI     │  │   RPC API    │  │  P2P Network   │  │
│  └─────────────┘  └──────────────┘  └────────────────┘  │
│         │                 │                   │          │
│  ┌──────────────────────────────────────────────────┐   │
│  │         Marketplace Manager                       │   │
│  │  ┌────────────────┐  ┌───────────────────────┐  │   │
│  │  │ State Channels │  │  Storage (Sled DB)    │  │   │
│  │  └────────────────┘  └───────────────────────┘  │   │
│  └──────────────────────────────────────────────────┘   │
│         │                                                │
│  ┌──────────────────────────────────────────────────┐   │
│  │         Tari L1 Client (Future)                   │   │
│  └──────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

## 🔧 System Requirements

- **OS**: Linux, macOS, or Windows (WSL)
- **Rust**: 1.70 or higher
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 10GB free space
- **Network**: Open ports for P2P and RPC

## 📦 Building from Source

```bash
# Clone repository
git clone https://github.com/tari-project/tari-l2
cd tari-l2

# Build
cargo build --release

# Run tests
cargo test

# Install (optional)
cargo install --path crates/l2-node
```

## 🎯 Usage Examples

### Start Single Node

```bash
./target/release/tari-l2-node init
./target/release/tari-l2-node start
```

### Create Channel Programmatically

```rust
use tari_l2_marketplace::MarketplaceManager;
use tari_l2_state_channel::ChannelConfig;

let manager = MarketplaceManager::new(storage, keypair);
let channel_id = manager.create_channel(config).await?;
manager.activate_channel(&channel_id).await?;
```

### RPC Call

```bash
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "get_channel_info",
    "params": {"channel_id": "abc123..."},
    "id": 1
  }'
```

## 📊 Testnet Status

### Implemented ✅
- State channel creation and management
- Multi-signature framework
- P2P gossipsub networking
- RPC JSON API
- Marketplace listings and orders
- Persistent storage
- CLI and configuration

### Partial ⚠️
- Multi-signature coordination (manual)
- Order fulfillment workflow
- Network peer discovery

### Not Implemented ❌
- Tari L1 integration (stubbed)
- Dispute resolution
- Advanced escrow
- Web dashboard
- Mobile SDK

## 🛠️ Development

### Run Tests

```bash
cargo test                    # All tests
cargo test --package tari-l2-common  # Specific package
```

### Run Demo

```bash
cargo run --package tari-l2-marketplace --example marketplace_demo
```

### Enable Debug Logging

```bash
RUST_LOG=debug ./target/release/tari-l2-node start
```

## 🤝 Contributing

We welcome contributions! Please see CONTRIBUTING.md for guidelines.

Areas needing help:
- L1 integration
- Web dashboard
- Additional tests
- Documentation
- Performance optimization

## ⚠️ Security

**THIS IS TESTNET SOFTWARE - DO NOT USE IN PRODUCTION**

Known limitations:
- Unencrypted private key storage
- Limited input validation
- No rate limiting
- Simplified fraud proofs
- Early stage networking

## 📝 License

MIT License - see LICENSE file

## 🔗 Links

- **Website**: https://tari.com
- **Documentation**: https://docs.tari.com
- **GitHub**: https://github.com/tari-project/tari-l2
- **Discord**: [Join Community]
- **Twitter**: [@tari]

## 🗺️ Roadmap

### v0.2.0 (Next Release)
- Automated multi-signature collection
- Functional L1 integration
- Enhanced RPC endpoints
- Web monitoring dashboard

### v0.3.0
- Advanced order workflows
- Dispute arbitration system
- State pruning
- Performance benchmarks

### v1.0.0 (Mainnet)
- Full security audit
- Production-ready L1 integration
- Complete documentation
- Mobile SDK
- Mainnet launch

## 👥 Team

Developed by the Tari community

## 📞 Support

- GitHub Issues for bugs
- Discord for general help
- Email: support@tari.com

---

**Current Version**: v0.1.0-testnet
**Network**: Testnet Alpha
**Status**: Active Development

*Built with ❤️ by the Tari community*
