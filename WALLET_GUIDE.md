# Tari Wallet Integration Guide

## Overview

TariMarket L2 includes a full Tari wallet implementation with:
- CipherSeed-based wallets (24-word Tari standard)
- Automatic wallet persistence
- Real UTXO scanning with private key
- Balance checking from blockchain

## Wallet Architecture

### CipherSeed Implementation

The wallet uses Tari's official CipherSeed standard:
- **24-word seed phrases** (Tari/BIP39 compatible)
- **Ristretto keypairs** (same as Tari console wallet)
- **Deterministic key derivation** from cipher seed entropy

### Wallet Storage

Wallets are automatically saved to the filesystem:

```
./data/
├── current_wallet.json        # Active wallet
└── wallet_{address}.json      # Backup per address
```

**Wallet File Format:**
```json
{
  "address": "🍗🌈🌕💻...",           # Emoji format
  "address_hex": "260208c4a16c...",  # Hex format (70 chars)
  "public_key": "08c4a16c61bc...",   # Public spend key
  "private_key": "f2228383faea...",  # Private spend key
  "seed_phrase": "gate federal tree relief govern hawk...",
  "created_at": "2025-10-03T08:33:09.994192770+00:00"
}
```

## API Reference

### Create Wallet

```bash
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"wallet_create","params":{},"id":1}'
```

### Import from Seed Phrase

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

### Check Balance (Auto-loads Saved Wallet)

```bash
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"get_l1_balance","params":{"address":"unused"},"id":1}'
```

### Check Balance with Specific Seed

```bash
curl -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"get_l1_balance",
    "params":{
      "address":"unused",
      "seed_phrase":"gate federal tree..."
    },
    "id":1
  }'
```

## Balance Scanning

Balance is calculated by:
1. Scanning last 1000 blocks from base node
2. Trying to decrypt each output with your private key
3. Summing all outputs you can decrypt

**Requires**: Base node must have `get_blocks` enabled in config.

## Security

**Testnet Only** - Not production-ready
- Wallets stored in plain JSON (no encryption)
- Add passphrase protection for production use
- Always backup your seed phrase
