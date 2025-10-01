#!/bin/bash
# Stop Tari L2 Testnet

set -e

DATA_DIR="./testnet_data"

echo "Stopping Tari L2 testnet nodes..."
echo ""

if [ ! -d "$DATA_DIR" ]; then
    echo "No testnet data found. Testnet may not be running."
    exit 0
fi

# Find and kill all node processes
for pid_file in "$DATA_DIR"/node*/node.pid; do
    if [ -f "$pid_file" ]; then
        PID=$(cat "$pid_file")
        NODE_NAME=$(dirname "$pid_file" | xargs basename)

        if ps -p $PID > /dev/null 2>&1; then
            echo "Stopping $NODE_NAME (PID: $PID)..."
            kill $PID
            sleep 1

            # Force kill if still running
            if ps -p $PID > /dev/null 2>&1; then
                echo "  Force stopping..."
                kill -9 $PID
            fi
        else
            echo "$NODE_NAME was not running"
        fi

        rm "$pid_file"
    fi
done

echo ""
echo "✓ All nodes stopped"
echo ""
echo "To clean up all data: rm -rf $DATA_DIR"
