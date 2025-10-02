#!/bin/bash

# Tari L2 Marketplace Web Interface Launcher

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NODE_PORT=18000
WEB_PORT=8080
WEB_DIR="web"

echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════╗"
echo "║  Tari L2 Marketplace Web Interface Launcher  ║"
echo "╚══════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if web directory exists
if [ ! -d "$WEB_DIR" ]; then
    echo -e "${RED}✗ Web directory not found: $WEB_DIR${NC}"
    echo "  Please run this script from the project root directory."
    exit 1
fi

echo -e "${GREEN}✓ Web directory found${NC}"

# Check if required files exist
required_files=("$WEB_DIR/index.html" "$WEB_DIR/app.js" "$WEB_DIR/style.css" "$WEB_DIR/test-data.js")
for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        echo -e "${RED}✗ Required file missing: $file${NC}"
        exit 1
    fi
done

echo -e "${GREEN}✓ All required files present${NC}"
echo

# Check if node is running
echo "Checking if Tari L2 node is running on port $NODE_PORT..."
if lsof -Pi :$NODE_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Node is running on port $NODE_PORT${NC}"

    # Try to get node info
    if command -v curl >/dev/null 2>&1; then
        echo -n "  Testing RPC connection... "
        if curl -s -X POST -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"get_node_info","params":{},"id":1}' \
            http://localhost:$NODE_PORT >/dev/null 2>&1; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${YELLOW}Warning: Could not connect to RPC${NC}"
        fi
    fi
else
    echo -e "${YELLOW}⚠  Node does not appear to be running on port $NODE_PORT${NC}"
    echo "   You can start it with: cargo run --release"
    echo "   The web interface will still launch, but won't be able to connect."
    echo
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo

# Check if web port is available
if lsof -Pi :$WEB_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠  Port $WEB_PORT is already in use${NC}"
    echo "   Finding an available port..."
    WEB_PORT=$((WEB_PORT + 1))
    while lsof -Pi :$WEB_PORT -sTCP:LISTEN -t >/dev/null 2>&1; do
        WEB_PORT=$((WEB_PORT + 1))
    done
    echo -e "${GREEN}✓ Using port $WEB_PORT instead${NC}"
fi

# Check for Python
PYTHON_CMD=""
if command -v python3 >/dev/null 2>&1; then
    PYTHON_CMD="python3"
elif command -v python >/dev/null 2>&1; then
    PYTHON_CMD="python"
else
    echo -e "${RED}✗ Python not found${NC}"
    echo "  Please install Python 3 to run the web server."
    exit 1
fi

echo -e "${GREEN}✓ Python found: $PYTHON_CMD${NC}"
echo

# Start the web server
echo -e "${BLUE}Starting web server on port $WEB_PORT...${NC}"
echo

cd "$WEB_DIR"

# Create a temp file for the PID
PIDFILE="/tmp/tari-l2-web-$WEB_PORT.pid"

# Function to cleanup on exit
cleanup() {
    echo
    echo -e "${YELLOW}Shutting down web server...${NC}"
    if [ -f "$PIDFILE" ]; then
        kill $(cat "$PIDFILE") 2>/dev/null || true
        rm "$PIDFILE"
    fi
    echo -e "${GREEN}✓ Web server stopped${NC}"
}

trap cleanup EXIT INT TERM

# Start the server in background
$PYTHON_CMD -m http.server $WEB_PORT >/dev/null 2>&1 &
SERVER_PID=$!
echo $SERVER_PID > "$PIDFILE"

# Wait a moment for server to start
sleep 1

# Check if server started successfully
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}✗ Failed to start web server${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Web server started successfully${NC}"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Web Interface Ready!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo -e "  ${BLUE}URL:${NC}          http://localhost:$WEB_PORT"
echo -e "  ${BLUE}Node RPC:${NC}     http://localhost:$NODE_PORT"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Try to open browser
if command -v xdg-open >/dev/null 2>&1; then
    echo "Opening browser..."
    xdg-open "http://localhost:$WEB_PORT" 2>/dev/null
elif command -v open >/dev/null 2>&1; then
    echo "Opening browser..."
    open "http://localhost:$WEB_PORT" 2>/dev/null
else
    echo "Please open your browser to: http://localhost:$WEB_PORT"
fi

echo
echo -e "${YELLOW}Press Ctrl+C to stop the web server${NC}"
echo

# Wait for the server process
wait $SERVER_PID
