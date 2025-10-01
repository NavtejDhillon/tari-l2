#!/bin/bash
# Tari L2 Testnet Deployment Script

set -e

echo "╔══════════════════════════════════════════════╗"
echo "║  Tari L2 Marketplace Testnet Deployment     ║"
echo "╚══════════════════════════════════════════════╝"
echo ""

# Configuration
NUM_NODES=${1:-3}
BASE_PORT=18000
BASE_P2P_PORT=9000
DATA_DIR="./testnet_data"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building project...${NC}"
cargo build --release

echo ""
echo -e "${GREEN}✓${NC} Build complete"
echo ""

# Clean up previous testnet data
if [ -d "$DATA_DIR" ]; then
    echo -e "${YELLOW}Cleaning up previous testnet data...${NC}"
    rm -rf "$DATA_DIR"
fi

mkdir -p "$DATA_DIR"

echo -e "${GREEN}Creating $NUM_NODES test nodes...${NC}"
echo ""

# Generate node configurations
for i in $(seq 1 $NUM_NODES); do
    NODE_DIR="$DATA_DIR/node$i"
    mkdir -p "$NODE_DIR"

    RPC_PORT=$((BASE_PORT + i - 1))
    P2P_PORT=$((BASE_P2P_PORT + i - 1))

    echo -e "${GREEN}Node $i:${NC}"
    echo "  Data dir: $NODE_DIR"
    echo "  RPC port: $RPC_PORT"
    echo "  P2P port: $P2P_PORT"

    # Create node config
    cat > "$NODE_DIR/config.toml" <<CONFIGEOF
data_dir = "$NODE_DIR/data"

[tari_node]
address = "127.0.0.1"
port = 18142

[network]
listen_addr = "/ip4/0.0.0.0/tcp/$P2P_PORT"
bootstrap_peers = []
max_peers = 50

[rpc]
listen_addr = "127.0.0.1"
port = $RPC_PORT
CONFIGEOF

    echo ""
done

# Create bootstrap list (each node connects to node1)
BOOTSTRAP_ADDR="/ip4/127.0.0.1/tcp/$BASE_P2P_PORT"

for i in $(seq 2 $NUM_NODES); do
    NODE_DIR="$DATA_DIR/node$i"
    # Update config with bootstrap peer - using different sed syntax for compatibility
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s|bootstrap_peers = \[\]|bootstrap_peers = [\"$BOOTSTRAP_ADDR\"]|" "$NODE_DIR/config.toml"
    else
        sed -i "s|bootstrap_peers = \[\]|bootstrap_peers = [\"$BOOTSTRAP_ADDR\"]|" "$NODE_DIR/config.toml"
    fi
done

echo -e "${GREEN}✓ Node configurations created${NC}"
echo ""
echo -e "${YELLOW}Starting nodes...${NC}"
echo ""

# Start nodes in background
for i in $(seq 1 $NUM_NODES); do
    NODE_DIR="$DATA_DIR/node$i"
    LOG_FILE="$NODE_DIR/node.log"

    echo -e "${GREEN}Starting node $i...${NC}"

    ./target/release/tari-l2-node --config "$NODE_DIR/config.toml" start > "$LOG_FILE" 2>&1 &
    NODE_PID=$!
    echo $NODE_PID > "$NODE_DIR/node.pid"

    echo "  PID: $NODE_PID"
    echo "  Logs: $LOG_FILE"
    echo ""

    # Give node time to start
    sleep 2
done

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Testnet Started Successfully        ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo "Node Status:"
for i in $(seq 1 $NUM_NODES); do
    RPC_PORT=$((BASE_PORT + i - 1))
    echo "  Node $i: http://127.0.0.1:$RPC_PORT"
done
echo ""
echo "To stop the testnet: ./scripts/stop_testnet.sh"
echo "To view logs: tail -f $DATA_DIR/node1/node.log"
echo ""
