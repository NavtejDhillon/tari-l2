#!/bin/bash
# Test all running nodes

echo "🧪 TESTING TARI L2 TESTNET NODES"
echo "================================="
echo ""

for port in 18000 18001 18002; do
    node_num=$((port - 18000 + 1))
    echo "Node $node_num (Port $port):"

    # Send RPC request
    response=$(echo '{"jsonrpc":"2.0","method":"list_channels","params":null,"id":1}' | timeout 2 nc 127.0.0.1 $port 2>/dev/null)

    if [ -n "$response" ]; then
        echo "  ✅ ONLINE - $response"
    else
        echo "  ❌ NO RESPONSE"
    fi
    echo ""
done

echo "================================="
echo "✅ Testnet Status Check Complete"
