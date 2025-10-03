# TariMarket L2 - Current Status

**Version**: 0.3.0-alpha
**Last Updated**: 2025-10-03
**Network**: Tari Esmeralda Testnet

## Completed Features

### Core Infrastructure
- State channel protocol with multi-party support
- P2P networking (libp2p + gossipsub)
- JSON-RPC API server
- Persistent storage (sled database)
- Marketplace state management
- Tari L1 client integration

### Wallet System
- Tari CipherSeed wallet (24-word seed phrases)
- Automatic wallet persistence (`./data/current_wallet.json`)
- Real UTXO scanning using wallet private key
- Encrypted output decryption to verify ownership
- Balance checking from blockchain
- Wallet import/export via seed phrases
- Multi-wallet support (specify seed_phrase parameter)

### Marketplace
- Create and broadcast listings (P2P)
- Browse all network listings
- Place orders with escrow
- Order tracking and status updates
- Cryptographic signatures (Ed25519)
- Signature verification on all nodes
- User profiles tied to wallet keys

### Web Interface
- Wallet creation/import UI
- Balance display (real blockchain scanning)
- Marketplace browsing
- Order management
- Profile management
- State channel UI
- Debug console

## Technical Stack

### Cryptography
- **Tari CipherSeed**: Official 24-word seed phrase standard
- **Ristretto**: EdDSA signatures on Curve25519
- **BLAKE3**: Fast cryptographic hashing
- **EncryptedData**: Tari's output encryption scheme

### Storage
- **sled**: Embedded key-value database
- **JSON files**: Wallet persistence (`./data/`)
- **Persistent channels**: State survives restarts

### Networking
- **libp2p**: Modular P2P networking
- **gossipsub**: Pub/sub message broadcasting
- **Noise protocol**: Encrypted connections

### Blockchain Integration
- **gRPC**: Communication with Tari base node
- **UTXO scanning**: Query blocks via `GetBlocks`
- **Output decryption**: Verify ownership with private key
- **Balance calculation**: Sum of owned outputs

## Project Structure

```
tari-l2/
├── crates/
│   ├── common/          # Shared types
│   ├── state-channel/   # Channel protocol
│   ├── marketplace/     # Business logic + Tari wallet
│   ├── p2p/             # libp2p networking
│   ├── rpc/             # JSON-RPC server + wallet storage
│   ├── l1-client/       # Tari integration + UTXO scanning
│   └── l2-node/         # Main binary
├── web/                 # Web interface
├── data/                # Runtime data (wallets, db)
│   ├── current_wallet.json
│   └── wallet_*.json
└── docs/                # Documentation
```

## Quick Start

```bash
# 1. Build
cargo build --release

# 2. Enable GetBlocks on Tari node
# Edit ~/.tari/esmeralda/config/config.toml
# Uncomment: "get_blocks" in grpc_server_allow_methods

# 3. Start Tari node
minotari_node --network esmeralda

# 4. Start L2 node
./target/release/tari-l2-node

# 5. Create wallet
curl -X POST http://localhost:18000 \
  -d '{"jsonrpc":"2.0","method":"wallet_create","params":{},"id":1}'

# 6. Check balance
curl -X POST http://localhost:18000 \
  -d '{"jsonrpc":"2.0","method":"get_l1_balance","params":{"address":"test"},"id":1}'
```

## Wallet Integration Details

### How It Works

1. **Wallet Creation**:
   - Generate CipherSeed (Tari standard)
   - Derive Ristretto keypair from seed entropy
   - Create Tari address (emoji + hex formats)
   - Save to `./data/current_wallet.json`

2. **Balance Checking**:
   - Connect to base node gRPC
   - Request blocks via `GetBlocks`
   - For each output: try to decrypt with private key
   - If decryption succeeds it's your output
   - Sum all owned outputs = balance

### Key Files

| File | Purpose |
|------|---------|
| `crates/marketplace/src/wallet.rs` | Tari CipherSeed wallet implementation |
| `crates/l1-client/src/lib.rs` | UTXO scanning + output decryption |
| `crates/rpc/src/api.rs` | Wallet RPC endpoints + persistence |
| `./data/current_wallet.json` | Active wallet storage |

### API Endpoints

| Method | Parameters | Returns |
|--------|-----------|---------|
| `wallet_create` | `{}` | New wallet + seed phrase |
| `wallet_import_seed` | `{seed_phrase}` | Imported wallet |
| `get_l1_balance` | `{address?, seed_phrase?}` | Balance in µT |

## Current Limitations

1. **Wallet Security**: Plain JSON storage (no encryption)
2. **Testnet Only**: Esmeralda testnet
3. **GetBlocks Required**: Base node must enable this method
4. **Scan Range**: Last 1000 blocks only
5. **No Hardware Wallet**: Software keys only

## Next Steps

### Short Term
- [ ] Encrypt wallet storage with passphrase
- [ ] Full blockchain scan option
- [ ] Wallet backup/restore UI
- [ ] Transaction history tracking

### Medium Term
- [ ] State checkpointing to L1
- [ ] Dispute resolution mechanism
- [ ] Fraud proof system
- [ ] Channel force-close

### Long Term
- [ ] Hardware wallet support (Ledger, Trezor)
- [ ] Multi-signature wallets
- [ ] Lightning-style routing
- [ ] Mobile app

## Documentation

- [README.md](README.md) - Main project documentation
- [WALLET_GUIDE.md](WALLET_GUIDE.md) - Detailed wallet guide
- [TESTING.md](TESTING.md) - Testing instructions
- [QUICK_START.md](QUICK_START.md) - Quick start guide

## Security Notes

### What's Secure
- Tari-standard cryptography (CipherSeed, Ristretto)
- Real UTXO scanning (not hardcoded balances)
- Private keys never transmitted
- Output ownership verified cryptographically

### What's NOT Secure (Production TODO)
- Wallet files stored in plain JSON
- No password protection
- Filesystem-based storage only
- No rate limiting on RPC

### Best Practices
- Backup seed phrase immediately
- Store seed phrase securely offline
- Never share private keys
- Use testnet only (not production ready)
- Encrypt wallets for production use

## Support

- **Issues**: https://github.com/yourusername/tari-l2/issues
- **Docs**: See [README.md](README.md)
- **Tari Discord**: https://discord.gg/tari

---

**Status**: Wallet Integration Complete | L1 Settlement In Progress
