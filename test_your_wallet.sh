#!/bin/bash

# Your wallet private key (first 64 chars of your string)
PRIVKEY="26033a0e40c80620643b89938daf6f420292fe09ab10d4fd45c8f56eded7dacd"

# Start node
./target/release/tari-l2-node > /tmp/tari-l2-node.log 2>&1 &
NODE_PID=$!
sleep 5

echo "=== TESTING YOUR WALLET IMPORT ==="
echo "Private key: $PRIVKEY"
echo ""

# Import wallet
RESULT=$(curl -s -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\":\"2.0\",
    \"method\":\"wallet_import_key\",
    \"params\":{\"private_key\":\"$PRIVKEY\"},
    \"id\":1
  }")

echo "Response:"
echo "$RESULT" | python3 -m json.tool

python3 << PYEOF
import json
result = '''$RESULT'''
data = json.loads(result)
if 'result' in data:
    r = data['result']
    print("\n=== YOUR WALLET ===")
    print(f"Emoji Address: {r['address'][:50]}...")
    print(f"Hex Address: {r['address_hex']}")
    print(f"Public Key: {r['public_key']}")
    print(f"\n=== FOR MINING ===")
    print(f"Use this hex address in your mining config:")
    print(r['address_hex'])
else:
    print("\nERROR:", data.get('error'))
PYEOF

# Cleanup
kill $NODE_PID 2>/dev/null
wait $NODE_PID 2>/dev/null
