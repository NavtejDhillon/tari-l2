#!/bin/bash
# Script to run Tari L2 node with Tari Esmeralda testnet

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Tari L2 Marketplace - Testnet Launch Script   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
echo ""

# Function to check if a process is running
check_process() {
    pgrep -f "$1" > /dev/null 2>&1
}

# Check if Tari testnet node is running
echo -e "${YELLOW}Checking for Tari Esmeralda testnet node...${NC}"
if check_process "minotari_node.*esmeralda"; then
    echo -e "${GREEN}✅ Tari testnet node is running${NC}"
else
    echo -e "${RED}⚠️  Tari testnet node not detected${NC}"
    echo ""
    echo "To start the Tari Esmeralda testnet node:"
    echo "  1. Download Tari from https://tari.com/downloads/"
    echo "  2. Run: minotari_node --network esmeralda"
    echo ""
    echo -e "${YELLOW}Starting L2 node in offline mode (mock L1 operations)...${NC}"
    echo ""
fi

# Check if we should build
if [ "$1" == "--skip-build" ]; then
    echo -e "${YELLOW}Skipping build step...${NC}"
else
    echo -e "${YELLOW}Building Tari L2 node...${NC}"
    if cargo build --release; then
        echo -e "${GREEN}✅ Build successful${NC}"
        echo ""
    else
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
fi

# Run the L2 node
echo -e "${BLUE}Starting Tari L2 node...${NC}"
echo ""
RUST_LOG=info cargo run --release --bin tari-l2-node
