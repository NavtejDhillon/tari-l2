# Tari L2 Marketplace Testnet Guide

## Overview

Welcome to the Tari L2 Marketplace testnet! This guide will help you set up and run a testnet node.

## Quick Start

### Prerequisites

- Rust 1.70+
- Linux/Mac OS (Windows via WSL)
- 4GB RAM minimum
- 10GB disk space

### Option 1: Automated Testnet Deployment (Recommended)

Deploy a local 3-node testnet:

```bash
# Build and start testnet
./scripts/deploy_testnet.sh 3

# Stop testnet
./scripts/stop_testnet.sh
```

### Option 2: Manual Single Node

```bash
# Build the project
cargo build --release

# Initialize configuration
./target/release/tari-l2-node init

# Edit config.toml as needed
nano config.toml

# Start the node
./target/release/tari-l2-node start
```

## Configuration

### Default Config (config.toml)

```toml
data_dir = "./data"

[tari_node]
address = "127.0.0.1"
port = 18142

[network]
listen_address = "/ip4/0.0.0.0/tcp/9000"
bootstrap_peers = []
max_peers = 50

[rpc]
listen_addr = "127.0.0.1"
port = 18000
```

### Configuration Options

#### Network Settings

- `listen_address`: P2P network listen address (libp2p multiaddr format)
- `bootstrap_peers`: List of initial peers to connect to
- `max_peers`: Maximum number of concurrent peer connections

#### RPC Settings

- `listen_addr`: RPC server bind address
- `port`: RPC server port

#### Storage

- `data_dir`: Directory for node data and database

### Connecting to Other Nodes

To join an existing testnet, add bootstrap peers to your config:

```toml
[network]
listen_address = "/ip4/0.0.0.0/tcp/9000"
bootstrap_peers = [
    "/ip4/192.168.1.100/tcp/9000/p2p/12D3KooW..."
]
```

## CLI Commands

### Initialize Node

```bash
tari-l2-node init
```

Creates a default configuration file.

### Start Node

```bash
tari-l2-node start
# or with custom config
tari-l2-node --config custom.toml start
```

### Set Log Level

```bash
tari-l2-node --log-level debug start
```

Levels: `trace`, `debug`, `info`, `warn`, `error`

### Show Version

```bash
tari-l2-node version
```

## RPC API

### Endpoints

All endpoints use JSON-RPC 2.0 format at `http://localhost:18000`

#### List Channels

```json
{
  "jsonrpc": "2.0",
  "method": "list_channels",
  "params": null,
  "id": 1
}
```

#### Get Channel Info

```json
{
  "jsonrpc": "2.0",
  "method": "get_channel_info",
  "params": {
    "channel_id": "hex_encoded_channel_id"
  },
  "id": 1
}
```

#### Get Balance

```json
{
  "jsonrpc": "2.0",
  "method": "get_balance",
  "params": {
    "channel_id": "hex_encoded_channel_id",
    "participant": "hex_encoded_public_key"
  },
  "id": 1
}
```

### Example: Using curl

```bash
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "list_channels",
    "params": null,
    "id": 1
  }'
```

## Testing the Marketplace

### 1. Create a Channel

```bash
# Use the demo to create test channels
cargo run --package tari-l2-marketplace --example marketplace_demo
```

### 2. Create Listings

See examples in `examples/marketplace_demo.rs` for creating listings programmatically.

### 3. Execute Orders

Multi-signature order execution flow:
1. Buyer creates order (state update)
2. All participants sign the update
3. Update is applied to channel state
4. Funds are transferred on order completion

## Network Topology

### Testnet Architecture

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   Node 1    │◄────►│   Node 2    │◄────►│   Node 3    │
│  :18000     │      │  :18001     │      │  :18002     │
│  P2P: 9000  │      │  P2P: 9001  │      │  P2P: 9002  │
└─────────────┘      └─────────────┘      └─────────────┘
       │                    │                     │
       └────────────────────┴─────────────────────┘
                    Gossipsub Network
```

### Peer Discovery

- Nodes use libp2p for P2P communication
- Gossipsub protocol for message broadcasting
- Bootstrap peers for initial network entry
- DHT for peer discovery (future enhancement)

## Monitoring

### View Node Logs

```bash
# Follow node logs
tail -f testnet_data/node1/node.log

# With custom data dir
tail -f ./data/node.log
```

### Check Node Status

```bash
# RPC health check
curl http://localhost:18000 -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"list_channels","id":1}'
```

### Metrics (Future)

Planned metrics endpoints:
- Active channels count
- Total transaction volume
- Peer count
- State update throughput

## Troubleshooting

### Port Already in Use

```
Error: Address already in use (os error 98)
```

Solution: Change ports in config.toml or stop conflicting services.

### Cannot Connect to Peers

1. Check firewall rules
2. Verify bootstrap peer addresses
3. Ensure nodes are on same network
4. Check logs for connection errors

### Database Corruption

```bash
# Backup data
mv ./data ./data.backup

# Reinitialize
./target/release/tari-l2-node init
./target/release/tari-l2-node start
```

### High Memory Usage

- Reduce `max_peers` in configuration
- Clear old channel history (future feature)
- Restart node periodically

## Development and Testing

### Run Unit Tests

```bash
cargo test
```

### Run Integration Tests

```bash
cargo test --test integration_test
```

### Run Example Demo

```bash
cargo run --package tari-l2-marketplace --example marketplace_demo
```

### Enable Debug Logging

```bash
RUST_LOG=debug tari-l2-node start
```

## Security Considerations

⚠️ **TESTNET ONLY - DO NOT USE IN PRODUCTION**

- Private keys are stored unencrypted
- No rate limiting implemented
- Limited input validation
- No DOS protection
- Simplified fraud proofs
- Test funds only

## Known Limitations

1. **Multi-Signature Collection**: State updates require all participants to sign, but coordination is manual
2. **L1 Integration**: Tari L1 connection is stubbed (not functional)
3. **Dispute Resolution**: Simplified implementation
4. **Order Escrow**: Basic implementation without advanced features
5. **Network Stability**: Early stage, may have connectivity issues
6. **Data Persistence**: No automatic state pruning

## Roadmap

### Current Version (v0.1.0 - Testnet Alpha)
- ✅ Basic state channels
- ✅ Simple marketplace operations
- ✅ P2P networking foundation
- ✅ CLI and configuration
- ⚠️ Manual multi-sig coordination

### Next Release (v0.2.0)
- Automated signature collection
- Functional L1 integration
- Enhanced RPC API
- Web dashboard
- Metrics and monitoring

### Future Releases
- Advanced order workflows
- Dispute arbitration
- State pruning
- Performance optimizations
- Mobile SDK

## Getting Help

- GitHub Issues: https://github.com/tari-project/tari-l2
- Discord: [Tari Community]
- Documentation: https://docs.tari.com/l2

## Contributing

See CONTRIBUTING.md for guidelines.

## License

MIT License - see LICENSE file
