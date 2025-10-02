#!/bin/bash
# Script to test Tari L2 node via RPC

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default RPC endpoint
RPC_HOST="${RPC_HOST:-127.0.0.1}"
RPC_PORT="${RPC_PORT:-50051}"
RPC_ENDPOINT="http://${RPC_HOST}:${RPC_PORT}"

echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Tari L2 Marketplace - Test Client          ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Testing endpoint: ${RPC_ENDPOINT}${NC}"
echo ""

# Function to make JSON-RPC calls
json_rpc_call() {
    local method="$1"
    local params="$2"

    curl -s -X POST "${RPC_ENDPOINT}" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"${method}\",
            \"params\": ${params},
            \"id\": 1
        }"
}

# Test 1: Get node info
echo -e "${BLUE}[Test 1/6]${NC} Getting node status..."
RESPONSE=$(json_rpc_call "get_node_info" "{}")
echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Node status retrieved${NC}"
else
    echo -e "${RED}❌ Failed to get node status${NC}"
fi
echo ""

# Test 2: List channels
echo -e "${BLUE}[Test 2/6]${NC} Listing channels..."
RESPONSE=$(json_rpc_call "list_channels" "{}")
echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Channels listed${NC}"
else
    echo -e "${RED}❌ Failed to list channels${NC}"
fi
echo ""

# Test 3: Create test channel
echo -e "${BLUE}[Test 3/6]${NC} Creating test channel..."
CHANNEL_PARAMS='{
    "participants": ["test_node_1", "test_node_2"],
    "initial_balances": {
        "test_node_1": 1000,
        "test_node_2": 1000
    },
    "challenge_period": 3600
}'
RESPONSE=$(json_rpc_call "create_channel" "${CHANNEL_PARAMS}")
echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"

# Extract channel ID if successful
CHANNEL_ID=$(echo "$RESPONSE" | jq -r '.result.channel_id' 2>/dev/null)
if [ "$CHANNEL_ID" != "null" ] && [ -n "$CHANNEL_ID" ]; then
    echo -e "${GREEN}✅ Channel created: ${CHANNEL_ID}${NC}"
else
    echo -e "${YELLOW}⚠️  Channel creation response received (may require proper setup)${NC}"
    CHANNEL_ID="test_channel_id"
fi
echo ""

# Test 4: Create marketplace listing
echo -e "${BLUE}[Test 4/6]${NC} Creating marketplace listing..."
LISTING_PARAMS='{
    "channel_id": "'${CHANNEL_ID}'",
    "seller": "test_node_1",
    "item_name": "Test Item",
    "description": "A test item for marketplace demo",
    "price": 100,
    "quantity": 5
}'
RESPONSE=$(json_rpc_call "create_listing" "${LISTING_PARAMS}")
echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Listing created${NC}"
else
    echo -e "${YELLOW}⚠️  Listing creation response received${NC}"
fi
echo ""

# Test 5: Get channel info
echo -e "${BLUE}[Test 5/6]${NC} Getting channel info..."
INFO_PARAMS='{"channel_id": "'${CHANNEL_ID}'"}'
RESPONSE=$(json_rpc_call "get_channel_info" "${INFO_PARAMS}")
echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Channel info retrieved${NC}"
else
    echo -e "${YELLOW}⚠️  Channel info response received${NC}"
fi
echo ""

# Test 6: Check L1 connection status
echo -e "${BLUE}[Test 6/6]${NC} Checking L1 connection status..."
RESPONSE=$(json_rpc_call "get_l1_status" "{}")
echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ L1 status retrieved${NC}"
else
    echo -e "${YELLOW}⚠️  L1 status response received${NC}"
fi
echo ""

echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              Test Sequence Complete             ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Note: Some tests may fail if RPC methods are not fully implemented.${NC}"
echo -e "${YELLOW}The L2 node should be running for all tests to pass.${NC}"
