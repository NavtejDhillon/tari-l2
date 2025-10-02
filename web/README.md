# Tari L2 Marketplace Web Interface

A modern, professional web interface for testing and interacting with the Tari L2 Marketplace node.

## Features

- **Real-time Status Monitoring**: Live connection status for node and L1 blockchain
- **State Channel Management**: Create, view, and manage payment channels
- **Marketplace Interface**: Create listings and browse products
- **Order Management**: Place and track orders through their lifecycle
- **Debug Tools**: Raw RPC interface and test data generators
- **Modern UI**: Dark/light theme toggle, responsive design, smooth animations

## Quick Start

### 1. Start Your Node

Make sure your Tari L2 node is running on port 18150:

```bash
cargo run --release
```

Or if using a custom port, update the RPC URL in `app.js`:

```javascript
const state = {
    rpc: new RPCClient('http://localhost:YOUR_PORT'),
    // ...
};
```

### 2. Start the Web Interface

Use the provided launcher script:

```bash
./start-web.sh
```

Or manually:

```bash
cd web
python3 -m http.server 8080
```

Then open your browser to: http://localhost:8080

## Available RPC Methods

### Node Management

- **get_node_info**: Get node public key and basic information
- **get_l1_status**: Check L1 blockchain connection status

### State Channels

- **list_channels**: List all state channels
- **create_channel**: Create a new payment channel
  ```json
  {
    "participant1": "64-char-hex-pubkey",
    "participant2": "64-char-hex-pubkey",
    "collateral": 1000000
  }
  ```
- **get_channel_info**: Get details for a specific channel
  ```json
  {
    "channel_id": "channel-id"
  }
  ```
- **transfer_in_channel**: Transfer funds within a channel
  ```json
  {
    "channel_id": "channel-id",
    "amount": 50000
  }
  ```
- **close_channel**: Close a payment channel
  ```json
  {
    "channel_id": "channel-id"
  }
  ```

### Marketplace

- **create_listing**: Create a new marketplace listing
  ```json
  {
    "title": "Product Name",
    "description": "Product description",
    "price": 100000,
    "category": "electronics",
    "seller_pubkey": "64-char-hex-pubkey"
  }
  ```
- **get_listings**: Get all marketplace listings
- **create_order**: Place an order for a listing
  ```json
  {
    "listing_id": "listing-id",
    "buyer_pubkey": "64-char-hex-pubkey"
  }
  ```
- **get_orders**: Get all orders
- **update_order_status**: Update order status
  ```json
  {
    "order_id": "order-id",
    "status": "confirmed|shipped|delivered|completed"
  }
  ```

## Testing Workflows

### Workflow 1: Create and Test a Payment Channel

1. Go to the **State Channels** tab
2. Click "Generate" buttons to create two random public keys
3. Enter a collateral amount (e.g., 1000000 µT)
4. Click "Create Channel"
5. View the channel in the list below
6. Click "Transfer" to move funds within the channel
7. Click "Close" to close the channel

### Workflow 2: Create and Sell a Product

1. Go to the **Marketplace** tab
2. Fill in the listing form:
   - Title: "My Product"
   - Description: "Product description"
   - Price: 100000 µT
   - Category: Choose from dropdown
   - Click "Generate" for seller key
3. Click "Create Listing"
4. View your listing in the grid
5. Click "Buy Now" to create an order
6. Go to **Orders** tab to track the order
7. Update order status through the lifecycle

### Workflow 3: Quick Testing with Generated Data

1. Use the **Quick Actions** panel at the top:
   - **Create Test Channel**: Instantly create a channel with random data
   - **Create Test Listing**: Generate a realistic product listing
   - **Refresh All Data**: Reload all data from the node
2. Go to the **Debug/Testing** tab:
   - **Generate Keypair**: Create random public keys
   - **Create 5 Test Channels**: Bulk create channels
   - **Create 10 Test Listings**: Populate marketplace with test data
   - **Create Test Order**: Generate an order from existing listings

### Workflow 4: Raw RPC Testing

1. Go to the **Debug/Testing** tab
2. Select an RPC method from the dropdown
3. Enter parameters as JSON in the text area
4. Click "Execute RPC Call"
5. View the response below
6. Check the console log for all RPC activity

## UI Features

### Status Indicators

- **Green dot**: Connected and working
- **Yellow dot**: Warning (e.g., L1 offline but node working)
- **Red dot**: Disconnected or error

### Theme Toggle

Click the moon/sun icon in the header to switch between light and dark themes. Your preference is saved in browser storage.

### Auto-refresh

The node and L1 status indicators automatically refresh every 5 seconds to show real-time connection state.

### Toast Notifications

Success, error, and info messages appear in the top-right corner:
- **Green**: Success
- **Red**: Error
- **Yellow**: Warning
- **Blue**: Info

### Console Log

The Debug tab includes a real-time console that logs all RPC calls with:
- Timestamp
- Method name
- Parameters
- Response/error
- Color-coded by status

## Troubleshooting

### "Failed to connect to node"

1. Verify your node is running: `cargo run`
2. Check the port (default: 18150)
3. Ensure no firewall is blocking the connection
4. Check browser console for CORS errors

### CORS Issues

If you see CORS errors in the browser console, your RPC server needs to allow cross-origin requests. Add CORS headers to your RPC server configuration.

### "No channels/listings/orders found"

This is normal for a fresh node. Use the quick actions or test data generators to populate with sample data.

### Port 8080 already in use

Change the port when starting the web server:

```bash
python3 -m http.server 8081
```

### RPC calls timeout

1. Check node logs for errors
2. Verify the RPC endpoint is responding: `curl http://localhost:18150`
3. Try increasing timeout in `app.js` if needed

## Browser Compatibility

Tested and working on:
- Chrome/Chromium 90+
- Firefox 88+
- Safari 14+
- Edge 90+

Requires a modern browser with ES6+ JavaScript support and the Crypto API for generating test keys.

## File Structure

```
web/
├── index.html       # Main HTML structure
├── style.css        # All styles and themes
├── app.js          # RPC client and UI logic
├── test-data.js    # Test data generators
├── README.md       # This file
```

## Development

### Modifying the RPC URL

Edit `app.js` line 3:

```javascript
const state = {
    rpc: new RPCClient('http://localhost:YOUR_PORT'),
    // ...
};
```

### Adding New RPC Methods

1. Add to the dropdown in `index.html`:
   ```html
   <option value="your_method">your_method</option>
   ```

2. Use in code:
   ```javascript
   const result = await state.rpc.call('your_method', params);
   ```

### Customizing Theme Colors

Edit CSS variables in `style.css`:

```css
:root {
    --primary: #2563eb;
    --success: #10b981;
    /* ... */
}
```

### Adding Test Data

Edit `test-data.js` to add more sample products or modify generation logic.

## Performance Notes

- The interface polls node status every 5 seconds
- Console log entries are kept in memory (clear periodically)
- Large numbers of items (100+ listings) may slow down rendering
- Use the "Clear Console" button to free up memory during long testing sessions

## Security Notes

This is a **development and testing interface**. Do not use in production with real funds without:

1. Implementing proper authentication
2. Using HTTPS for all connections
3. Validating all user inputs server-side
4. Rate limiting RPC calls
5. Securing the RPC endpoint

## License

Same license as the Tari L2 project.

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review the node logs
3. Check the browser console for errors
4. Refer to the main Tari L2 documentation
