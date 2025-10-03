#!/bin/bash

# Your wallet address (first 70 chars only - the actual Tari address)
ADDRESS="26032683c1495b7877cf3e49e333a59f5ea8ad59bde89f73dabf1b4c74f85a988f7eda"

# Start the L2 node if not running
./target/release/tari-l2-node > /tmp/tari-l2-node.log 2>&1 &
NODE_PID=$!
sleep 5

echo "========================================="
echo "CHECKING BALANCE FOR YOUR WALLET"
echo "========================================="
echo "Address: $ADDRESS"
echo ""

# Check balance via RPC
RESULT=$(curl -s -X POST http://localhost:18000 \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\":\"2.0\",
    \"method\":\"get_l1_balance\",
    \"params\":{\"address\":\"$ADDRESS\"},
    \"id\":1
  }")

echo "RPC Response:"
echo "$RESULT" | python3 -m json.tool

echo ""
echo "========================================="
python3 << PYEOF
import json
result = '''$RESULT'''
try:
    data = json.loads(result)
    if 'result' in data and data['result']:
        balance_microtari = data['result'].get('balance', 0)
        balance_tari = balance_microtari / 1000000
        source = data['result'].get('source', 'unknown')
        
        print(f"Balance: {balance_microtari} µT ({balance_tari:.6f} XTM)")
        print(f"Source: {source}")
        
        if balance_microtari == 0:
            print("\n⚠️  Balance is 0")
            print("This could mean:")
            print("1. Base node is not connected")
            print("2. UTXO scanning not implemented yet")
            print("3. Blocks not confirmed yet")
            print("4. Mining rewards sent to different address")
    elif 'error' in data:
        print(f"❌ ERROR: {data['error']['message']}")
    else:
        print("❌ Unexpected response format")
except Exception as e:
    print(f"❌ Error parsing response: {e}")
PYEOF

echo ""
echo "========================================="
echo "Checking L2 node logs for errors..."
echo "========================================="
tail -20 /tmp/tari-l2-node.log | grep -i "balance\|error\|l1"

# Cleanup
kill $NODE_PID 2>/dev/null
wait $NODE_PID 2>/dev/null
