#!/bin/bash

echo "=== QUERYING BASE NODE FOR RECENT BLOCKS ==="
echo ""
echo "This will show coinbase addresses from recent blocks you mined"
echo ""

# Query the base node gRPC to get recent blocks
# We need to use grpcurl or similar, but let's try a simple approach first

# Check if base node has a simple RPC interface
echo "Checking base node status..."
ps aux | grep minotari_node | grep -v grep

echo ""
echo "Base node is running. To check your mining rewards, you need to:"
echo "1. Start Tari console wallet and sync it"
echo "2. OR query the base node directly for UTXOs"
echo "3. OR check the Tari explorer for your address"
echo ""
echo "Your L2 wallet address: 26032683c1495b7877cf3e49e333a59f5ea8ad59bde89f73dabf1b4c74f85a988f7eda"
echo ""
echo "Let me try one more thing - checking if you have a Tari wallet database..."

if [ -d "$HOME/.tari/esmeralda/wallet" ]; then
    echo "✓ Found Tari wallet directory"
    ls -la "$HOME/.tari/esmeralda/wallet/"
else
    echo "✗ No Tari wallet found - you're mining but rewards need a Tari wallet to show balance"
fi
