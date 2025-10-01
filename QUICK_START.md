# Tari L2 Marketplace - Quick Start Guide

## Launch Testnet in 5 Minutes

### Step 1: Build

```bash
cargo build --release
```

### Step 2: Deploy Testnet

```bash
./scripts/deploy_testnet.sh 3
```

This creates 3 interconnected nodes:
- Node 1: RPC on :18000, P2P on :9000
- Node 2: RPC on :18001, P2P on :9001
- Node 3: RPC on :18002, P2P on :9002

### Step 3: Verify Nodes Running

```bash
# Check logs
tail -f testnet_data/node1/node.log

# Test RPC
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"list_channels","id":1}'
```

### Step 4: Stop Testnet

```bash
./scripts/stop_testnet.sh
```

## Manual Single Node Setup

```bash
# Initialize config
./target/release/tari-l2-node init

# Start node
./target/release/tari-l2-node start

# Or with debug logging
./target/release/tari-l2-node --log-level debug start
```

## What's Working

✅ **State Channels** - Create, activate, and manage payment channels
✅ **P2P Networking** - Nodes discover and communicate via libp2p
✅ **Marketplace Operations** - Create listings, orders, transfers
✅ **Multi-signature Framework** - State updates require all participant signatures
✅ **Persistence** - All data stored in Sled database
✅ **RPC API** - JSON-RPC interface for external clients
✅ **CLI** - Full command-line interface for node management

## Testing the Marketplace

Run the interactive demo:

```bash
cargo run --package tari-l2-marketplace --example marketplace_demo
```

This demonstrates:
- Creating a channel between buyer and seller
- Checking balances
- Creating product listings
- Generating state updates

## Configuration

Edit `config.toml` to customize:

```toml
data_dir = "./data"

[tari_node]
address = "127.0.0.1"
port = 18142

[network]
listen_addr = "/ip4/0.0.0.0/tcp/9000"
bootstrap_peers = []
max_peers = 50

[rpc]
listen_addr = "127.0.0.1"
port = 18000
```

## Network Ports

- **RPC**: 18000-18002 (default 18000)
- **P2P**: 9000-9002 (default 9000)
- **Tari L1**: 18142 (if enabled)

## Next Steps

1. Read `TESTNET_GUIDE.md` for detailed documentation
2. Review `TESTING.md` for testing procedures
3. Check `README.md` for architecture overview
4. Join the community (links in main README)

## Troubleshooting

**Build fails**: Ensure Rust 1.70+ is installed
```bash
rustc --version
rustup update
```

**Ports in use**: Change ports in config.toml

**Can't connect peers**: Check firewall and ensure bootstrap peers are correct

## Security Warning

⚠️ **TESTNET ONLY** - This is alpha software. Do not use with real funds.

## Getting Help

- GitHub Issues
- Community Discord
- Documentation at docs.tari.com

---

**Ready for Production?** Not yet. See TESTNET_GUIDE.md for limitations and roadmap.
