#!/bin/bash

# Start the node in background
echo "Starting L2 node..."
./target/release/tari-l2-node > /tmp/tari-l2-node.log 2>&1 &
NODE_PID=$!

# Wait for node to start
sleep 5

# Test wallet creation
echo "Testing wallet_create..."
RESULT=$(curl -s -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"wallet_create",
    "params":{},
    "id":1
  }')

echo "$RESULT" | python3 -m json.tool

# Extract and analyze with python
python3 << PYEOF
import json
import sys

result = '''$RESULT'''
try:
    data = json.loads(result)
    if 'result' in data:
        r = data['result']
        emoji_addr = r.get('address', '')
        hex_addr = r.get('address_hex', '')
        pubkey = r.get('public_key', '')
        privkey = r.get('private_key', '')
        seed = r.get('seed_phrase', '')
        
        print("\n=== ANALYSIS ===")
        print(f"Emoji address length: {len(emoji_addr)} chars")
        print(f"Hex address length: {len(hex_addr)} chars")
        print(f"Public key length: {len(pubkey)} chars")
        print(f"Private key length: {len(privkey)} chars")
        print(f"Seed phrase words: {len(seed.split())}")
        
        print("\n=== FOR MINING ===")
        print(f"HEX ADDRESS: {hex_addr}")
        print(f"\nFirst 50 chars of emoji: {emoji_addr[:50]}")
    else:
        print("ERROR:", data.get('error', 'Unknown error'))
except Exception as e:
    print(f"Error parsing: {e}")
    print(f"Raw result: {result[:200]}")
PYEOF

# Cleanup
kill $NODE_PID 2>/dev/null
wait $NODE_PID 2>/dev/null
