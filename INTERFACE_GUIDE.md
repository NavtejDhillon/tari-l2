# Tari L2 Marketplace - How to Use It

## Current Status: v0.1.0 (Alpha Testnet)

Currently, there's **NO graphical web interface** - but you have several ways to interact with your marketplace!

## ✅ What You Can Do RIGHT NOW

### 1. Use the Interactive CLI

```bash
cargo run --package tari-l2-marketplace --example marketplace_cli
```

This gives you a menu-driven interface to:
- ✅ List all channels
- ✅ Get channel information
- ✅ Check balances
- ⚠️ Create listings (coming in v0.2.0)
- ⚠️ Create orders (coming in v0.2.0)
- ⚠️ Transfer funds (coming in v0.2.0)

### 2. Run the Demo Application

```bash
cargo run --package tari-l2-marketplace --example marketplace_demo
```

This demonstrates the full workflow:
- Creates a channel between buyer and seller
- Checks balances
- Creates a product listing
- Generates state updates
- Shows channel info

### 3. Use Direct RPC Calls

#### List All Channels
```bash
echo '{"jsonrpc":"2.0","method":"list_channels","params":null,"id":1}' | nc 127.0.0.1 18000
```

#### Get Channel Info
```bash
echo '{"jsonrpc":"2.0","method":"get_channel_info","params":{"channel_id":"YOUR_CHANNEL_ID"},"id":1}' | nc 127.0.0.1 18000
```

#### Check Balance
```bash
echo '{"jsonrpc":"2.0","method":"get_balance","params":{"channel_id":"YOUR_CHANNEL_ID","participant":"PUBLIC_KEY"},"id":1}' | nc 127.0.0.1 18000
```

### 4. Write Your Own Client

Use any programming language that supports JSON-RPC:

**Python Example:**
```python
import socket
import json

def call_rpc(method, params=None):
    request = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    }

    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(('127.0.0.1', 18000))
    sock.send((json.dumps(request) + '\n').encode())

    response = sock.recv(4096).decode()
    sock.close()

    return json.loads(response)

# List channels
result = call_rpc("list_channels")
print(json.dumps(result, indent=2))
```

**JavaScript/Node.js Example:**
```javascript
const net = require('net');

function callRPC(method, params = null) {
    return new Promise((resolve, reject) => {
        const client = net.createConnection({ port: 18000 }, () => {
            const request = JSON.stringify({
                jsonrpc: "2.0",
                method: method,
                params: params,
                id: 1
            }) + '\n';

            client.write(request);
        });

        client.on('data', (data) => {
            resolve(JSON.parse(data.toString()));
            client.end();
        });

        client.on('error', reject);
    });
}

// Usage
callRPC('list_channels').then(console.log);
```

## 🚧 Coming in v0.2.0 (Next Release)

### Web Dashboard
- **Full web UI** for marketplace operations
- **Real-time updates** of channel status
- **Visual channel explorer**
- **Listing browser**
- **Order management interface**
- **Balance tracker**

### Enhanced CLI
- **Create channels** from CLI
- **Manage listings** interactively
- **Place and track orders**
- **Transfer funds** between participants

### REST API
- In addition to JSON-RPC
- RESTful endpoints
- WebSocket support for real-time updates

## 📋 Current Workflow (Manual)

To actually use the marketplace right now, you need to:

1. **Create channels programmatically:**
   - Edit and run the demo
   - Or write your own Rust code using the marketplace crate

2. **Sign state updates:**
   - Currently requires manual multi-sig coordination
   - Each participant must sign updates independently

3. **Query via RPC:**
   - Use CLI or direct RPC calls to check status

## 💡 Quick Start Recipes

### Recipe 1: Create and Monitor a Channel

```bash
# Terminal 1: Run demo to create channel
cargo run --example marketplace_demo

# Terminal 2: Monitor node logs
tail -f testnet_data/node1/node.log

# Terminal 3: Query via CLI
cargo run --example marketplace_cli
# Choose option 1 to list channels
```

### Recipe 2: Multi-Node Testing

```bash
# Create channel in node 1
echo '...' | nc 127.0.0.1 18000

# Query from node 2
echo '{"jsonrpc":"2.0","method":"list_channels","id":1}' | nc 127.0.0.1 18001

# Query from node 3
echo '{"jsonrpc":"2.0","method":"list_channels","id":1}' | nc 127.0.0.1 18002
```

## 🎯 Roadmap

### v0.1.0 (Current)
- ✅ RPC API
- ✅ CLI demo
- ✅ Programmatic access

### v0.2.0 (Next - ~4 weeks)
- 🔨 Web dashboard
- 🔨 Enhanced CLI
- 🔨 Auto multi-sig collection
- 🔨 REST API

### v0.3.0 (Future - ~8 weeks)
- 🔮 Mobile app
- 🔮 Advanced order workflows
- 🔮 Dispute resolution UI
- 🔮 Analytics dashboard

### v1.0.0 (Mainnet - ~16 weeks)
- 🚀 Production web UI
- 🚀 Full mobile SDK
- 🚀 Merchant tools
- 🚀 Buyer protection

## 🔗 Useful Links

- **RPC API Docs**: See TESTNET_GUIDE.md
- **Examples**: `/examples/` directory
- **Test Scripts**: `/scripts/` directory

## 📞 Get Help

- **Issues**: https://github.com/tari-project/tari-l2/issues
- **Discord**: [Tari Community]
- **Docs**: https://docs.tari.com/l2

---

**TL;DR**: Right now, use the **CLI** (`cargo run --example marketplace_cli`) or **demo** (`cargo run --example marketplace_demo`) to interact with your marketplace. A full web UI is coming in v0.2.0!
