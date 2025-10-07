// RPC Client Class
class RPCClient {
    constructor(url = 'http://192.168.86.208:18000') {
        this.url = url;
        this.requestId = 0;
    }

    async call(method, params = {}) {
        const id = ++this.requestId;
        const request = {
            jsonrpc: '2.0',
            method: method,
            params: params,
            id: id
        };

        // Log to console
        logToConsole(method, params, null, 'request');

        try {
            const response = await fetch(this.url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(request)
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();

            if (data.error) {
                logToConsole(method, params, data.error, 'error');
                throw new Error(data.error.message || 'RPC Error');
            }

            logToConsole(method, params, data.result, 'success');
            return data.result;
        } catch (error) {
            logToConsole(method, params, { message: error.message }, 'error');
            throw error;
        }
    }
}

// Global state
const state = {
    rpc: new RPCClient(),
    nodeInfo: null,
    l1Status: null,
    channels: [],
    listings: [],
    orders: [],
    autoRefreshInterval: null,
    wallet: null,
    profile: null
};

// Initialize app
document.addEventListener('DOMContentLoaded', async () => {
    // Check for wallet first
    const walletData = localStorage.getItem('tari_wallet');
    if (!walletData) {
        window.location.href = 'wallet.html';
        return;
    }

    // Load wallet into state
    try {
        state.wallet = JSON.parse(walletData);

        // Load profile
        const profileData = localStorage.getItem('tari_profile');
        if (profileData) {
            state.profile = JSON.parse(profileData);
        }

        updateWalletDisplay();
    } catch (error) {
        console.error('Failed to load wallet:', error);
        localStorage.removeItem('tari_wallet');
        window.location.href = 'wallet.html';
        return;
    }

    initTheme();
    initTabs();
    initForms();
    initQuickActions();
    initDebugTab();
    await initializeNode();
    startAutoRefresh();
});

// Theme Management
function initTheme() {
    const savedTheme = localStorage.getItem('theme') || 'light';
    document.documentElement.setAttribute('data-theme', savedTheme);
    updateThemeIcon(savedTheme);

    document.getElementById('themeToggle').addEventListener('click', () => {
        const currentTheme = document.documentElement.getAttribute('data-theme');
        const newTheme = currentTheme === 'light' ? 'dark' : 'light';
        document.documentElement.setAttribute('data-theme', newTheme);
        localStorage.setItem('theme', newTheme);
        updateThemeIcon(newTheme);
    });
}

function updateThemeIcon(theme) {
    const icon = document.querySelector('.theme-icon');
    icon.textContent = theme === 'light' ? '🌙' : '☀️';
}

// Tab Management
function initTabs() {
    const tabButtons = document.querySelectorAll('.tab-btn');
    const tabPanes = document.querySelectorAll('.tab-pane');

    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const tabName = button.getAttribute('data-tab');

            // Update active states
            tabButtons.forEach(btn => btn.classList.remove('active'));
            tabPanes.forEach(pane => pane.classList.remove('active'));

            button.classList.add('active');
            document.getElementById(`${tabName}-tab`).classList.add('active');
        });
    });
}

// Initialize Node Connection
async function initializeNode() {
    try {
        // Get node info
        const nodeInfo = await state.rpc.call('get_node_info');
        state.nodeInfo = nodeInfo;

        // Update UI
        updateNodeStatus(true);
        if (nodeInfo.public_key) {
            document.querySelector('.pubkey-text').textContent = nodeInfo.public_key;
        }

        // Get L1 status
        try {
            const l1Status = await state.rpc.call('get_l1_status');
            state.l1Status = l1Status;
            updateL1Status(l1Status.connected || false);
            if (l1Status.network) {
                document.getElementById('networkBadge').textContent = l1Status.network;
            }
        } catch (error) {
            updateL1Status(false);
        }

        // Load initial data
        await loadChannels();
        await loadListings();
        await loadOrders();

        showToast('Connected to node successfully', 'success');
    } catch (error) {
        updateNodeStatus(false);
        updateL1Status(false);
        showToast('Failed to connect to node: ' + error.message, 'error');
    }
}

// Status Updates
function updateNodeStatus(connected) {
    const statusDot = document.querySelector('#nodeStatus .status-dot');
    if (connected) {
        statusDot.classList.remove('disconnected');
        statusDot.classList.add('connected');
    } else {
        statusDot.classList.remove('connected');
        statusDot.classList.add('disconnected');
    }
}

function updateL1Status(connected) {
    const statusDot = document.querySelector('#l1Status .status-dot');
    if (connected) {
        statusDot.classList.remove('disconnected', 'warning');
        statusDot.classList.add('connected');
    } else {
        statusDot.classList.remove('connected');
        statusDot.classList.add('warning');
    }
}

// Form Initialization
function initForms() {
    // Create Channel Form
    document.getElementById('createChannelForm').addEventListener('submit', handleCreateChannel);
    document.getElementById('genKey1').addEventListener('click', () => {
        document.getElementById('participant1').value = generateKeyPair();
    });
    document.getElementById('genKey2').addEventListener('click', () => {
        document.getElementById('participant2').value = generateKeyPair();
    });

    // Note: Create Listing Form is now handled via modal (see openCreateListingModal)

    // Refresh buttons
    document.getElementById('refreshChannels').addEventListener('click', loadChannels);
    document.getElementById('refreshListings').addEventListener('click', loadListings);
    document.getElementById('refreshOrders').addEventListener('click', loadOrders);
}

// Quick Actions
function initQuickActions() {
    document.getElementById('quickCreateChannel').addEventListener('click', async () => {
        const key1 = generateKeyPair();
        const key2 = generateKeyPair();
        const collateral = Math.floor(Math.random() * 1000000) + 100000;

        try {
            showLoading(true);
            await state.rpc.call('create_channel', {
                participant1: key1,
                participant2: key2,
                collateral: collateral
            });
            showToast('Test channel created successfully', 'success');
            await loadChannels();
        } catch (error) {
            showToast('Failed to create channel: ' + error.message, 'error');
        } finally {
            showLoading(false);
        }
    });

    document.getElementById('quickCreateListing').addEventListener('click', async () => {
        const listing = generateTestListing();
        try {
            showLoading(true);
            await state.rpc.call('create_listing', listing);
            showToast('Test listing created successfully', 'success');
            await loadListings();
        } catch (error) {
            showToast('Failed to create listing: ' + error.message, 'error');
        } finally {
            showLoading(false);
        }
    });

    document.getElementById('quickRefresh').addEventListener('click', async () => {
        await Promise.all([
            loadChannels(),
            loadListings(),
            loadOrders()
        ]);
        showToast('Data refreshed', 'info');
    });

    document.getElementById('quickClearConsole').addEventListener('click', () => {
        document.getElementById('consoleLog').innerHTML = '';
        showToast('Console cleared', 'info');
    });
}

// Channel Management
async function handleCreateChannel(e) {
    e.preventDefault();

    const participant1 = document.getElementById('participant1').value;
    const participant2 = document.getElementById('participant2').value;
    const collateral = parseInt(document.getElementById('collateral').value);

    try {
        showLoading(true);
        await state.rpc.call('create_channel', {
            participant1,
            participant2,
            collateral
        });

        showToast('Channel created successfully', 'success');
        document.getElementById('createChannelForm').reset();
        await loadChannels();
    } catch (error) {
        showToast('Failed to create channel: ' + error.message, 'error');
    } finally {
        showLoading(false);
    }
}

async function loadChannels() {
    try {
        const channels = await state.rpc.call('list_channels');
        state.channels = channels || [];
        renderChannels();
    } catch (error) {
        document.getElementById('channelsList').innerHTML =
            '<div class="empty-state"><div class="empty-state-icon">🔗</div><div class="empty-state-text">No channels found</div></div>';
    }
}

function renderChannels() {
    const container = document.getElementById('channelsList');

    if (state.channels.length === 0) {
        container.innerHTML =
            '<div class="empty-state"><div class="empty-state-icon">🔗</div><div class="empty-state-text">No channels yet</div><div class="empty-state-subtext">Create your first state channel</div></div>';
        return;
    }

    container.innerHTML = state.channels.map(channel => `
        <div class="item-card">
            <div class="item-header">
                <span class="item-id">${channel.id || 'N/A'}</span>
                <span class="item-status ${channel.status === 'active' ? 'status-active' : 'status-closed'}">
                    ${channel.status || 'unknown'}
                </span>
            </div>
            <div class="item-details">
                <div class="item-detail">
                    <span class="label">Participant 1:</span>
                    <span class="value">${truncateHex(channel.participant1)}</span>
                </div>
                <div class="item-detail">
                    <span class="label">Participant 2:</span>
                    <span class="value">${truncateHex(channel.participant2)}</span>
                </div>
                <div class="item-detail">
                    <span class="label">Balance:</span>
                    <span class="value">${formatMicroTari(channel.balance || 0)}</span>
                </div>
            </div>
            <div class="item-actions">
                <button class="btn-secondary btn-sm" onclick="transferInChannel('${channel.id}')">Transfer</button>
                <button class="btn-secondary btn-sm" onclick="closeChannel('${channel.id}')">Close</button>
            </div>
        </div>
    `).join('');

    // Also update channel selector in listing form
    updateChannelSelector();
}

function updateChannelSelector() {
    const select = document.getElementById('listingChannelId');
    if (!select) return;

    // Keep the current selection if any
    const currentValue = select.value;

    // Clear and repopulate
    select.innerHTML = '<option value="">Select a channel</option>' +
        state.channels.map(channel =>
            `<option value="${channel.id}">${truncateHex(channel.id)} (${channel.status})</option>`
        ).join('');

    // Restore selection if still valid
    if (currentValue && state.channels.find(c => c.id === currentValue)) {
        select.value = currentValue;
    }
}

async function transferInChannel(channelId) {
    const amount = prompt('Enter amount to transfer (µT):');
    if (!amount) return;

    try {
        showLoading(true);
        await state.rpc.call('transfer_in_channel', {
            channel_id: channelId,
            amount: parseInt(amount)
        });
        showToast('Transfer successful', 'success');
        await loadChannels();
    } catch (error) {
        showToast('Transfer failed: ' + error.message, 'error');
    } finally {
        showLoading(false);
    }
}

async function closeChannel(channelId) {
    if (!confirm('Are you sure you want to close this channel?')) return;

    try {
        showLoading(true);
        await state.rpc.call('close_channel', { channel_id: channelId });
        showToast('Channel closed successfully', 'success');
        await loadChannels();
    } catch (error) {
        showToast('Failed to close channel: ' + error.message, 'error');
    } finally {
        showLoading(false);
    }
}

// Marketplace Management
async function handleCreateListing(e) {
    e.preventDefault();

    if (!state.wallet) {
        showToast('Wallet not loaded', 'error');
        return;
    }

    if (!state.wallet.public_key) {
        showToast('Wallet missing public key. Please create a new wallet.', 'error');
        console.error('Wallet state:', state.wallet);
        return;
    }

    const listing = {
        seller_pubkey: state.wallet.public_key,  // Use public_key (32 bytes hex) for RPC calls
        title: document.getElementById('listingTitle').value,
        description: document.getElementById('listingDescription').value,
        price: parseInt(document.getElementById('listingPrice').value),
        category: document.getElementById('listingCategory')?.value || 'other'
    };

    // DEBUG: Log what we're sending
    console.log('Creating listing with seller_pubkey:', listing.seller_pubkey);
    console.log('Wallet object:', state.wallet);

    try {
        showLoading(true);
        await state.rpc.call('create_listing', listing);

        showToast('Listing created successfully', 'success');
        document.getElementById('createListingForm').reset();

        // Restore seller display after reset
        const sellerDisplay = document.getElementById('sellerDisplay');
        if (sellerDisplay && state.profile) {
            sellerDisplay.textContent = `${state.profile.name}${state.profile.location ? ' - ' + state.profile.location : ''}`;
        }

        await loadListings();
    } catch (error) {
        showToast('Failed to create listing: ' + error.message, 'error');
    } finally {
        showLoading(false);
    }
}

async function loadListings() {
    try {
        const listings = await state.rpc.call('get_listings');
        state.listings = listings || [];
        renderListings();
    } catch (error) {
        document.getElementById('listingsList').innerHTML =
            '<div class="empty-state"><div class="empty-state-icon">📦</div><div class="empty-state-text">No listings found</div></div>';
    }
}

function getSellerDisplay(seller) {
    // Check if this is the current user's listing
    if (state.wallet && (seller === state.wallet.address || seller === state.wallet.public_key)) {
        if (state.profile && state.profile.name) {
            return `${state.profile.name} (You)`;
        }
        return 'You';
    }

    // For other sellers, just show truncated pubkey (could be enhanced with a profile lookup system later)
    return truncateHex(seller);
}

function renderListings() {
    const container = document.getElementById('listingsList');

    // Apply filters
    const filteredListings = state.listings.filter(listing => {
        const matchesCategory = state.activeCategory === 'all' || listing.category === state.activeCategory;
        const matchesSearch = !state.searchQuery ||
            listing.title.toLowerCase().includes(state.searchQuery.toLowerCase()) ||
            listing.description.toLowerCase().includes(state.searchQuery.toLowerCase());
        return matchesCategory && matchesSearch;
    });

    if (filteredListings.length === 0 && state.listings.length === 0) {
        container.innerHTML =
            '<div class="empty-state"><div class="empty-state-icon">📦</div><div class="empty-state-text">No listings yet</div><div class="empty-state-subtext">Create your first marketplace listing</div></div>';
        return;
    }

    if (filteredListings.length === 0) {
        container.innerHTML =
            '<div class="empty-state"><div class="empty-state-icon">📦</div><div class="empty-state-text">No listings found</div><div class="empty-state-subtext">Try adjusting your filters</div></div>';
        return;
    }

    // Get category icon
    const getCategoryIcon = (category) => {
        const icons = {
            electronics: '💻',
            clothing: '👕',
            books: '📚',
            services: '🛠️',
            digital: '💾',
            other: '📦'
        };
        return icons[category] || '📦';
    };

    container.innerHTML = filteredListings.map(listing => {
        // DEBUG: Log listing data
        console.log('Rendering listing:', listing);

        const sellerDisplay = getSellerDisplay(listing.seller);
        const categoryIcon = getCategoryIcon(listing.category);

        return `
        <div class="listing-card" onclick="showListingDetails('${listing.id}')">
            <div class="listing-image">
                ${categoryIcon}
            </div>
            <div class="listing-content">
                <div class="listing-header">
                    <span class="listing-category">${listing.category || 'uncategorized'}</span>
                    <div class="listing-title">${escapeHtml(listing.title)}</div>
                </div>
                <div class="listing-description">${escapeHtml(listing.description)}</div>
                <div class="listing-footer">
                    <div>
                        <div class="listing-price">${(listing.price || 0).toLocaleString()} XTM</div>
                    </div>
                    <div class="listing-seller" title="${listing.seller}">${sellerDisplay}</div>
                </div>
            </div>
        </div>
        `;
    }).join('');
}

function showListingDetails(listingId) {
    // For now, just trigger buy - can be enhanced to show a detail modal
    createOrder(listingId);
}

async function createOrder(listingId) {
    // Prevent buying own listing
    const listing = state.listings.find(l => l.id === listingId);
    if (listing && state.wallet && 
        (listing.seller === state.wallet.public_key || listing.seller === state.wallet.address)) {
        showToast('You cannot purchase your own listing', 'error');
        return;
    }
    
    const buyerPubkey = prompt('Enter buyer public key (or leave empty to generate):');
    const pubkey = buyerPubkey || generateKeyPair();

    try {
        showLoading(true);
        await state.rpc.call('create_order', {
            listing_id: listingId,
            buyer_pubkey: pubkey
        });
        showToast('Order created successfully', 'success');
        await loadOrders();
    } catch (error) {
        showToast('Failed to create order: ' + error.message, 'error');
    } finally {
        showLoading(false);
    }
}

// Orders Management
async function loadOrders() {
    try {
        const orders = await state.rpc.call('get_orders');
        state.orders = orders || [];
        renderOrders();
    } catch (error) {
        document.getElementById('ordersList').innerHTML =
            '<div class="empty-state"><div class="empty-state-icon">🛒</div><div class="empty-state-text">No orders found</div></div>';
    }
}

function renderOrders() {
    const container = document.getElementById('ordersList');

    if (state.orders.length === 0) {
        container.innerHTML =
            '<div class="empty-state"><div class="empty-state-icon">🛒</div><div class="empty-state-text">No orders yet</div><div class="empty-state-subtext">Orders will appear here after purchases</div></div>';
        return;
    }

    container.innerHTML = state.orders.map(order => `
        <div class="item-card">
            <div class="item-header">
                <span class="item-id">${order.id || 'N/A'}</span>
                <span class="item-status status-${order.status || 'pending'}">${order.status || 'pending'}</span>
            </div>
            <div class="item-details">
                <div class="item-detail">
                    <span class="label">Listing:</span>
                    <span class="value">${order.listing_title || order.listing_id}</span>
                </div>
                <div class="item-detail">
                    <span class="label">Buyer:</span>
                    <span class="value">${truncateHex(order.buyer_pubkey)}</span>
                </div>
                <div class="item-detail">
                    <span class="label">Seller:</span>
                    <span class="value">${truncateHex(order.seller_pubkey)}</span>
                </div>
                <div class="item-detail">
                    <span class="label">Amount:</span>
                    <span class="value">${formatMicroTari(order.amount)}</span>
                </div>
            </div>
            <div class="item-actions">
                <button class="btn-secondary btn-sm" onclick="updateOrderStatus('${order.id}', 'confirmed')">Confirm</button>
                <button class="btn-secondary btn-sm" onclick="updateOrderStatus('${order.id}', 'shipped')">Ship</button>
                <button class="btn-secondary btn-sm" onclick="updateOrderStatus('${order.id}', 'completed')">Complete</button>
            </div>
        </div>
    `).join('');
}

async function updateOrderStatus(orderId, status) {
    try {
        showLoading(true);
        await state.rpc.call('update_order_status', {
            order_id: orderId,
            status: status
        });
        showToast(`Order status updated to ${status}`, 'success');
        await loadOrders();
    } catch (error) {
        showToast('Failed to update order: ' + error.message, 'error');
    } finally {
        showLoading(false);
    }
}

// Debug Tab
function initDebugTab() {
    // RPC Form
    document.getElementById('rpcForm').addEventListener('submit', async (e) => {
        e.preventDefault();

        const method = document.getElementById('rpcMethod').value;
        const paramsText = document.getElementById('rpcParams').value;

        try {
            const params = JSON.parse(paramsText);
            const result = await state.rpc.call(method, params);
            document.getElementById('rpcResponse').textContent = JSON.stringify(result, null, 2);
        } catch (error) {
            document.getElementById('rpcResponse').textContent = 'Error: ' + error.message;
        }
    });

    // Test data generators
    document.getElementById('genKeypair').addEventListener('click', () => {
        const keypair = generateKeyPair();
        document.getElementById('testOutput').textContent = JSON.stringify({ public_key: keypair }, null, 2);
    });

    document.getElementById('genMultipleChannels').addEventListener('click', async () => {
        try {
            showLoading(true);
            const results = [];
            for (let i = 0; i < 5; i++) {
                const key1 = generateKeyPair();
                const key2 = generateKeyPair();
                const collateral = Math.floor(Math.random() * 1000000) + 100000;

                const result = await state.rpc.call('create_channel', {
                    participant1: key1,
                    participant2: key2,
                    collateral: collateral
                });
                results.push(result);
            }
            document.getElementById('testOutput').textContent = JSON.stringify(results, null, 2);
            showToast('Created 5 test channels', 'success');
            await loadChannels();
        } catch (error) {
            showToast('Failed to create channels: ' + error.message, 'error');
        } finally {
            showLoading(false);
        }
    });

    document.getElementById('genMultipleListings').addEventListener('click', async () => {
        try {
            showLoading(true);
            const results = [];
            for (let i = 0; i < 10; i++) {
                const listing = generateTestListing();
                const result = await state.rpc.call('create_listing', listing);
                results.push(result);
            }
            document.getElementById('testOutput').textContent = JSON.stringify(results, null, 2);
            showToast('Created 10 test listings', 'success');
            await loadListings();
        } catch (error) {
            showToast('Failed to create listings: ' + error.message, 'error');
        } finally {
            showLoading(false);
        }
    });

    document.getElementById('genTestOrder').addEventListener('click', async () => {
        if (state.listings.length === 0) {
            showToast('Create some listings first', 'warning');
            return;
        }

        try {
            showLoading(true);
            const listing = state.listings[Math.floor(Math.random() * state.listings.length)];
            const buyerPubkey = generateKeyPair();

            const result = await state.rpc.call('create_order', {
                listing_id: listing.id,
                buyer_pubkey: buyerPubkey
            });

            document.getElementById('testOutput').textContent = JSON.stringify(result, null, 2);
            showToast('Created test order', 'success');
            await loadOrders();
        } catch (error) {
            showToast('Failed to create order: ' + error.message, 'error');
        } finally {
            showLoading(false);
        }
    });

    document.getElementById('clearConsole').addEventListener('click', () => {
        document.getElementById('consoleLog').innerHTML = '';
    });
}

// Console Logging
function logToConsole(method, params, response, type) {
    const consoleLog = document.getElementById('consoleLog');
    const timestamp = new Date().toLocaleTimeString();

    const entry = document.createElement('div');
    entry.className = 'console-entry';

    const typeClass = type === 'error' ? 'console-error' : type === 'success' ? 'console-success' : '';

    entry.innerHTML = `
        <div class="console-timestamp">[${timestamp}]</div>
        <div class="console-method ${typeClass}">${type.toUpperCase()}: ${method}</div>
        ${params && Object.keys(params).length > 0 ? `<div class="console-params">Params: ${JSON.stringify(params)}</div>` : ''}
        ${response ? `<div class="console-response ${typeClass}">Response: ${JSON.stringify(response)}</div>` : ''}
    `;

    consoleLog.appendChild(entry);

    // Auto-scroll if enabled
    if (document.getElementById('autoScroll')?.checked) {
        consoleLog.scrollTop = consoleLog.scrollHeight;
    }
}

// Auto Refresh
function startAutoRefresh() {
    // Initial status check
    checkNodeStatus();

    // Refresh every 5 seconds
    state.autoRefreshInterval = setInterval(async () => {
        await checkNodeStatus();
    }, 5000);
}

async function checkNodeStatus() {
    try {
        const nodeInfo = await state.rpc.call('get_node_info');
        updateNodeStatus(true);

        try {
            const l1Status = await state.rpc.call('get_l1_status');
            updateL1Status(l1Status.connected || false);
        } catch {
            updateL1Status(false);
        }
    } catch {
        updateNodeStatus(false);
        updateL1Status(false);
    }
}

// Utility Functions
function formatMicroTari(amount) {
    return `${(amount || 0).toLocaleString()} XTM`;
}

function truncateHex(hex, start = 8, end = 8) {
    if (!hex || hex === 'undefined' || hex === 'null') return 'Unknown';
    if (hex.length <= start + end) return hex;
    return `${hex.substring(0, start)}...${hex.substring(hex.length - end)}`;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showLoading(show) {
    const overlay = document.getElementById('loadingOverlay');
    if (show) {
        overlay.classList.remove('hidden');
    } else {
        overlay.classList.add('hidden');
    }
}

function showToast(message, type = 'info') {
    const container = document.getElementById('toastContainer');
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;

    container.appendChild(toast);

    setTimeout(() => {
        toast.style.animation = 'slideIn 0.3s ease-out reverse';
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

// Wallet Management
function updateWalletDisplay() {
    if (!state.wallet) return;

    const walletAddress = document.getElementById('walletAddress');
    if (walletAddress) {
        // Show profile name if available, otherwise show truncated address
        if (state.profile && state.profile.name) {
            walletAddress.textContent = state.profile.name;
            walletAddress.title = `${state.profile.name} (${state.wallet.address || state.wallet.public_key})`;
        } else {
            walletAddress.textContent = truncateHex(state.wallet.address || state.wallet.public_key, 6, 6);
            walletAddress.title = state.wallet.address || state.wallet.public_key;
        }
    }
}

function showWalletDetails() {
    if (!state.wallet) return;

    const details = `
Wallet Address: ${state.wallet.address || state.wallet.public_key}

Click OK to copy to clipboard.
    `;

    if (confirm(details)) {
        navigator.clipboard.writeText(state.wallet.address || state.wallet.public_key);
        showToast('Wallet address copied to clipboard', 'success');
    }
}

function exportWallet() {
    if (!state.wallet) return;

    const exportData = {
        address: state.wallet.address || state.wallet.public_key,
        private_key: state.wallet.private_key,
        seed_phrase: state.wallet.seed_phrase
    };

    const dataStr = JSON.stringify(exportData, null, 2);
    const dataUri = 'data:application/json;charset=utf-8,' + encodeURIComponent(dataStr);

    const exportFileDefaultName = `tari-wallet-${Date.now()}.json`;

    const linkElement = document.createElement('a');
    linkElement.setAttribute('href', dataUri);
    linkElement.setAttribute('download', exportFileDefaultName);
    linkElement.click();

    showToast('Wallet exported successfully', 'success');
}

function logoutWallet() {
    if (!confirm('Are you sure you want to logout? Make sure you have backed up your wallet!')) {
        return;
    }

    localStorage.removeItem('tari_wallet');
    localStorage.removeItem('tari_profile');
    window.location.href = 'wallet.html';
}

// Profile Management
function openProfileModal() {
    if (!state.wallet) {
        showToast('No wallet loaded', 'error');
        return;
    }

    // Pre-fill form with current profile data
    if (state.profile) {
        document.getElementById('editProfileName').value = state.profile.name || '';
        document.getElementById('editProfileLocation').value = state.profile.location || '';
        document.getElementById('editProfileBio').value = state.profile.bio || '';
        document.getElementById('editProfileEmail').value = state.profile.email || '';
    }

    document.getElementById('profileWalletAddress').textContent = state.wallet.address || state.wallet.public_key;
    document.getElementById('profileWalletAddressHex').textContent = state.wallet.address_hex || '';
    document.getElementById('profileModal').style.display = 'flex';
}

function closeProfileModal() {
    document.getElementById('profileModal').style.display = 'none';
}

function saveProfile() {
    const name = document.getElementById('editProfileName').value.trim();
    if (!name) {
        showToast('Display name is required', 'error');
        return;
    }

    const location = document.getElementById('editProfileLocation').value.trim();
    const bio = document.getElementById('editProfileBio').value.trim();
    const email = document.getElementById('editProfileEmail').value.trim();

    // Update profile
    const updatedProfile = {
        name,
        location: location || null,
        bio: bio || null,
        email: email || null,
        public_key: state.wallet.address || state.wallet.public_key,
        rating: state.profile?.rating || 0,
        transactions_completed: state.profile?.transactions_completed || 0
    };

    // Save to state and localStorage
    state.profile = updatedProfile;
    localStorage.setItem('tari_profile', JSON.stringify(updatedProfile));

    // Update wallet object with profile
    state.wallet.profile = updatedProfile;
    localStorage.setItem('tari_wallet', JSON.stringify(state.wallet));

    // Update UI
    updateWalletDisplay();

    // Update seller display if on marketplace tab
    const sellerDisplay = document.getElementById('sellerDisplay');
    if (sellerDisplay) {
        sellerDisplay.textContent = `${state.profile.name}${state.profile.location ? ' - ' + state.profile.location : ''}`;
    }

    closeProfileModal();
    showToast('Profile updated successfully', 'success');
}

// ==================== Marketplace Modal Functions ====================

function openCreateListingModal() {
    const modal = document.getElementById('createListingModal');
    modal.style.display = 'flex';

    // Update seller display
    const sellerDisplay = document.getElementById('sellerDisplay');
    if (sellerDisplay && state.profile) {
        sellerDisplay.textContent = `${state.profile.name}${state.profile.location ? ' - ' + state.profile.location : ''}`;
    } else if (sellerDisplay && state.wallet) {
        sellerDisplay.textContent = truncateHex(state.wallet.address, 8, 8);
    }
}

function closeCreateListingModal() {
    const modal = document.getElementById('createListingModal');
    modal.style.display = 'none';
    document.getElementById('createListingForm').reset();
}

async function submitCreateListing() {
    const form = document.getElementById('createListingForm');
    if (!form.checkValidity()) {
        form.reportValidity();
        return;
    }

    const title = document.getElementById('listingTitle').value;
    const description = document.getElementById('listingDescription').value;
    const price = parseInt(document.getElementById('listingPrice').value);
    const category = document.getElementById('listingCategory').value;

    try {
        showLoading(true);
        await state.rpc.call('create_listing', {
            title,
            description,
            price,
            category,
            seller: state.wallet?.public_key || state.wallet?.address
        });

        showToast('Listing created successfully', 'success');
        closeCreateListingModal();
        await loadListings();
    } catch (error) {
        showToast('Failed to create listing: ' + error.message, 'error');
    } finally {
        showLoading(false);
    }
}

// Close modal when clicking outside
document.addEventListener('click', (e) => {
    const modal = document.getElementById('createListingModal');
    if (e.target === modal) {
        closeCreateListingModal();
    }
});

// ==================== Enhanced Listing Rendering ====================

// Initialize state for filters
if (!state.activeCategory) state.activeCategory = 'all';
if (!state.searchQuery) state.searchQuery = '';

// ==================== Search and Filter Functions ====================

function initMarketplaceFilters() {
    // Category filter buttons
    const categoryBtns = document.querySelectorAll('.category-btn');
    categoryBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            // Update active state
            categoryBtns.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');

            // Update filter
            state.activeCategory = btn.dataset.category;
            renderListings();
        });
    });

    // Search input
    const searchInput = document.getElementById('searchListings');
    if (searchInput) {
        searchInput.addEventListener('input', (e) => {
            state.searchQuery = e.target.value;
            renderListings();
        });
    }
}

// Initialize marketplace filters when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initMarketplaceFilters);
} else {
    setTimeout(initMarketplaceFilters, 100);
}

// ==================== Balance Display ====================

async function updateBalanceDisplay() {
    const balanceAmountEl = document.getElementById('balanceAmount');
    const balanceDisplayEl = document.getElementById('balanceDisplay');

    if (!balanceAmountEl || !balanceDisplayEl) return;

    try {
        // Try to get L1 balance using wallet address
        if (state.wallet && state.wallet.address_hex) {
            try {
                // Show scanning progress
                balanceAmountEl.innerHTML = '<span class="loading-text" style="font-size: 11px;">Connecting...</span>';
                balanceAmountEl.title = 'Scanning blockchain for UTXOs';

                await new Promise(resolve => setTimeout(resolve, 100));

                balanceAmountEl.innerHTML = '<span class="loading-text" style="font-size: 11px;">Connecting to base node...</span>';

                await new Promise(resolve => setTimeout(resolve, 100));

                balanceAmountEl.innerHTML = '<span class="loading-text" style="font-size: 11px;">Querying blocks...</span>';

                const startTime = Date.now();
                const l1Result = await state.rpc.call('get_l1_balance', {
                    address: state.wallet.address_hex
                });

                const elapsed = Date.now() - startTime;

                // Show scanning progress
                if (elapsed < 500) {
                    balanceAmountEl.innerHTML = '<span class="loading-text" style="font-size: 11px;">Scanning UTXOs...</span>';
                    await new Promise(resolve => setTimeout(resolve, 300));
                }

                balanceAmountEl.innerHTML = '<span class="loading-text" style="font-size: 11px;">Decrypting outputs...</span>';
                await new Promise(resolve => setTimeout(resolve, 200));

                balanceAmountEl.innerHTML = '<span class="loading-text" style="font-size: 11px;">Calculating balance...</span>';
                await new Promise(resolve => setTimeout(resolve, 200));

                if (l1Result && l1Result.balance !== undefined) {
                    balanceAmountEl.textContent = formatMicroTari(l1Result.balance);
                    balanceAmountEl.title = 'L1 Balance (from base node UTXO scan, includes mining rewards)';
                    balanceDisplayEl.style.cursor = 'pointer';
                    balanceDisplayEl.onclick = showBalanceInfo;
                    return;
                }
            } catch (l1Error) {
                console.log('L1 balance query failed:', l1Error);
                // Continue to show 0 balance or L2 balance
            }
        }

        // Fallback to L2 channel balance
        const channels = await state.rpc.call('list_channels');

        if (!channels || channels.length === 0) {
            balanceAmountEl.innerHTML = '<span class="loading-text">0 XTM</span>';
            balanceAmountEl.title = 'No L2 channels or L1 balance';
            balanceDisplayEl.style.cursor = 'pointer';
            balanceDisplayEl.onclick = showBalanceInfo;
            return;
        }

        // Calculate total balance across all L2 channels
        let totalBalance = 0;
        for (const channel of channels) {
            if (channel.balance) {
                totalBalance += channel.balance;
            }
        }

        balanceAmountEl.textContent = formatMicroTari(totalBalance);
        balanceAmountEl.title = 'L2 Channel Balance';
        balanceDisplayEl.style.cursor = 'pointer';
        balanceDisplayEl.onclick = showBalanceInfo;
    } catch (error) {
        console.error('Failed to fetch balance:', error);
        balanceAmountEl.innerHTML = '<span class="loading-text">Error</span>';
        balanceDisplayEl.style.cursor = 'pointer';
        balanceDisplayEl.onclick = showBalanceInfo;
    }
}

async function showBalanceInfo() {
    try {
        // Get L1 status and node info
        const l1Status = await state.rpc.call('get_l1_status');
        const nodeInfo = await state.rpc.call('get_node_info');
        const isL1Connected = l1Status && l1Status.connected;

        // Get wallet info
        const walletAddress = state.wallet?.address || state.wallet?.public_key || 'Not loaded';
        const walletName = state.profile?.name || 'Anonymous';

        const message = `💰 Wallet & Balance Information

${isL1Connected ? '✅ Connected to Tari L1 (Esmeralda Testnet)' : '⚠️ L1 Not Connected'}

👤 Your L2 Marketplace Wallet:
• Name: ${walletName}
• L2 Address: ${walletAddress}

⚠️ IMPORTANT: This is an L2-only address!
   This address is for L2 marketplace operations.
   It cannot receive L1 XTM directly.

🔹 Balance shown: ${state.listings?.length || 0} listings, ${state.channels?.length || 0} channels
🔹 Network: ${l1Status?.network || 'Unknown'}
🔹 L1 Endpoint: ${l1Status?.endpoint || 'Not configured'}

🔑 Your L2 Node Public Key:
${nodeInfo?.public_key || 'Loading...'}

📝 How to get testnet XTM and use this marketplace:

${isL1Connected ?
`Since L1 is connected:

1. Install Tari Aurora wallet or minotari_console_wallet
2. Create a proper L1 wallet address
3. Get testnet XTM from Tari community/faucet
4. Send XTM to your L1 wallet
5. Use L1 wallet to lock funds in L2 channels
6. Trade on this L2 marketplace

💡 Two types of addresses:
   • L1 Wallet Address: For receiving testnet XTM (base58 format)
   • L2 Address (this one): For marketplace operations (hex format)

   You need BOTH:
   - L1 wallet to hold testnet XTM
   - L2 address (shown above) for marketplace trading` :
`L1 is not connected. To enable real funds:

1. Add [l1] config in config.toml
2. Connect to a Tari base node
3. Install a Tari L1 wallet
4. Get testnet XTM from faucet

Currently channels use mock collateral.`}

🌐 Get Tari Wallet & Testnet XTM:
• Download Tari Aurora: https://www.tari.com/downloads/
• Tari Discord: https://discord.gg/tari
• Ask for testnet XTM in Discord #testnet channel

⚠️ TESTNET ONLY - No real value`;

        alert(message);
    } catch (error) {
        console.error('Error fetching balance info:', error);
        alert('Balance Information\n\nError loading wallet status. Please check your connection.');
    }
}

// Update balance periodically
setInterval(updateBalanceDisplay, 10000); // Every 10 seconds
setTimeout(updateBalanceDisplay, 2000);  // Initial load after 2s
