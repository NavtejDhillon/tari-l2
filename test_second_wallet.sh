#!/bin/bash

PRIVKEY="260324a664227eeec80169306a9ded3017b028fcaf3cfb36f3a3350f11d5cc38"

./target/release/tari-l2-node > /tmp/tari-l2-node.log 2>&1 &
NODE_PID=$!
sleep 5

echo "=== TESTING SECOND WALLET IMPORT ==="
RESULT=$(curl -s -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\":\"2.0\",
    \"method\":\"wallet_import_key\",
    \"params\":{\"private_key\":\"$PRIVKEY\"},
    \"id\":1
  }")

echo "$RESULT" | python3 -m json.tool

python3 << PYEOF
import json
result = '''$RESULT'''
data = json.loads(result)
if 'result' in data and data['result']:
    r = data['result']
    print("\n✓ WALLET IMPORTED SUCCESSFULLY!")
    print(f"\nHex Address for mining:")
    print(r['address_hex'])
else:
    print("\n✗ IMPORT FAILED")
    if 'error' in data:
        print(f"Error: {data['error']['message']}")
PYEOF

kill $NODE_PID 2>/dev/null
wait $NODE_PID 2>/dev/null
