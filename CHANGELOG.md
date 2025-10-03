# Changelog

## [0.3.0-alpha] - 2025-10-03

### Major Features - Tari Wallet Integration

#### Wallet System
- Full Tari CipherSeed wallet implementation
  - 24-word seed phrases (Tari/BIP39 standard)
  - Ristretto keypair generation
  - Deterministic key derivation from cipher seed entropy
  - Compatible with Tari console wallet

- Automatic wallet persistence
  - Wallets saved to `./data/current_wallet.json`
  - Individual backup files per address (`wallet_{address}.json`)
  - Timestamp tracking for wallet creation
  - Auto-load on balance queries

- Real UTXO scanning from blockchain
  - Scan last 1000 blocks via base node gRPC
  - Decrypt encrypted outputs using wallet's private key
  - Verify output ownership cryptographically
  - Calculate real balance

- Standard Tari address generation
  - Standard Tari address format (70-char hex)
  - Emoji address format for display (35 emojis)
  - Network byte prefix (2602/2603 for Esmeralda)

#### API Enhancements
- `wallet_create`: Create new wallet with seed phrase
- `wallet_import_seed`: Import from 24-word seed
- `get_l1_balance`: Check balance with auto-loaded wallet
- Support for explicit seed_phrase/private_key parameters

#### Technical Implementation
- Added `tari_transaction_components` dependency for EncryptedData
- Added `tari_crypto` for Ristretto keys
- Implemented `EncryptedData::decrypt_data()` for output scanning
- Added `CompressedCommitment` parsing
- Added `chrono` for wallet timestamps

#### Files Modified
- `crates/marketplace/src/wallet.rs` - Tari wallet implementation
- `crates/l1-client/src/lib.rs` - UTXO scanning logic
- `crates/rpc/src/api.rs` - Wallet persistence & RPC endpoints
- `crates/l1-client/Cargo.toml` - Added Tari dependencies
- `crates/rpc/Cargo.toml` - Added chrono dependency

### Documentation Updates
- Updated `README.md` with wallet integration details
- Created `WALLET_GUIDE.md` - Comprehensive wallet guide
- Created `STATUS.md` - Current project status
- Created `CHANGELOG.md` - This file
- Removed outdated documentation files

### Configuration
- Base node must enable `get_blocks` in gRPC config
- L2 node automatically creates `./data/` directory
- Wallet files are JSON formatted for easy inspection

### Bug Fixes
- Fixed UTXO scanning to use proper wallet keys (not count all outputs)
- Fixed gRPC stream handling with `tokio_stream::StreamExt`
- Fixed output field access (`output_type`, `commitment`, etc.)
- Resolved circular dependency between l1-client and marketplace

### Known Limitations
- Wallets stored in plain JSON (no encryption)
- Scans last 1000 blocks only (not full history)
- Requires base node `get_blocks` permission
- Testnet only (Esmeralda)

### Security Notes
- Uses official Tari crypto libraries
- Private keys stored locally, never transmitted
- Output ownership verified cryptographically
- **Production TODO**: Add wallet encryption with passphrase

---

## [0.2.0-alpha] - Previous Release

### Features
- P2P marketplace with libp2p
- State channels
- Listing broadcasts
- Order management
- User profiles
- Web interface

---

## Future Releases

### [0.4.0] - Planned
- Encrypted wallet storage
- Full blockchain scanning
- Transaction history
- Wallet backup UI

### [0.5.0] - Planned
- L1 state checkpointing
- Dispute resolution
- Fraud proofs
- Channel force-close
