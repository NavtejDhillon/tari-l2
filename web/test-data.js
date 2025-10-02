// Test Data Generators

// Generate random hex string of specified length (in bytes)
function generateRandomHex(bytes = 32) {
    const array = new Uint8Array(bytes);
    crypto.getRandomValues(array);
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
}

// Generate a random keypair (just public key for testing)
function generateKeyPair() {
    return generateRandomHex(32);
}

// Sample product data
const sampleProducts = [
    {
        title: "Vintage Mechanical Keyboard",
        description: "Cherry MX Blue switches, RGB backlight, excellent condition",
        category: "electronics",
        priceRange: [50000, 200000]
    },
    {
        title: "Premium Noise-Cancelling Headphones",
        description: "Wireless, 30-hour battery life, studio quality sound",
        category: "electronics",
        priceRange: [150000, 350000]
    },
    {
        title: "Smart Watch Series X",
        description: "Fitness tracking, heart rate monitor, waterproof",
        category: "electronics",
        priceRange: [200000, 500000]
    },
    {
        title: "Limited Edition Sneakers",
        description: "Size 10, never worn, original box included",
        category: "clothing",
        priceRange: [100000, 300000]
    },
    {
        title: "Designer Leather Jacket",
        description: "Genuine leather, size M, classic style",
        category: "clothing",
        priceRange: [250000, 600000]
    },
    {
        title: "Programming Course Bundle",
        description: "Learn Rust, Go, and Python - 100+ hours of content",
        category: "digital",
        priceRange: [50000, 150000]
    },
    {
        title: "Digital Art Collection",
        description: "10 high-resolution abstract artworks",
        category: "digital",
        priceRange: [30000, 100000]
    },
    {
        title: "Web Development Services",
        description: "Full-stack development, responsive design, modern frameworks",
        category: "services",
        priceRange: [500000, 2000000]
    },
    {
        title: "Logo Design Package",
        description: "Professional logo design with 3 revisions included",
        category: "services",
        priceRange: [100000, 300000]
    },
    {
        title: "Rare First Edition Book",
        description: "Classic literature, excellent condition, collector's item",
        category: "books",
        priceRange: [150000, 500000]
    },
    {
        title: "Tech Book Bundle",
        description: "5 books on blockchain, cryptography, and distributed systems",
        category: "books",
        priceRange: [80000, 200000]
    },
    {
        title: "Gaming Mouse",
        description: "16000 DPI, programmable buttons, RGB lighting",
        category: "electronics",
        priceRange: [40000, 120000]
    },
    {
        title: "4K Webcam",
        description: "Professional streaming quality, autofocus, built-in mic",
        category: "electronics",
        priceRange: [80000, 200000]
    },
    {
        title: "Handmade Ceramic Mug Set",
        description: "Set of 4, unique designs, microwave safe",
        category: "other",
        priceRange: [30000, 80000]
    },
    {
        title: "Vintage Vinyl Records",
        description: "Collection of 20 classic rock albums, good condition",
        category: "other",
        priceRange: [100000, 300000]
    },
    {
        title: "Standing Desk",
        description: "Electric height adjustment, memory presets, sturdy build",
        category: "other",
        priceRange: [200000, 500000]
    },
    {
        title: "Ergonomic Office Chair",
        description: "Lumbar support, adjustable everything, mesh back",
        category: "other",
        priceRange: [150000, 400000]
    },
    {
        title: "Photography Lightroom Presets",
        description: "50+ professional presets for various styles",
        category: "digital",
        priceRange: [20000, 60000]
    },
    {
        title: "Music Production Sample Pack",
        description: "1000+ samples, loops, and one-shots for electronic music",
        category: "digital",
        priceRange: [40000, 120000]
    },
    {
        title: "Personal Training Sessions",
        description: "10-session package, customized workout plans",
        category: "services",
        priceRange: [300000, 800000]
    }
];

// Generate a random test listing
function generateTestListing() {
    const product = sampleProducts[Math.floor(Math.random() * sampleProducts.length)];
    const price = Math.floor(
        Math.random() * (product.priceRange[1] - product.priceRange[0]) + product.priceRange[0]
    );

    return {
        title: product.title,
        description: product.description,
        price: price,
        category: product.category,
        seller_pubkey: generateKeyPair()
    };
}

// Generate multiple test listings
function generateMultipleListings(count = 10) {
    const listings = [];
    for (let i = 0; i < count; i++) {
        listings.push(generateTestListing());
    }
    return listings;
}

// Generate test channel data
function generateTestChannel() {
    return {
        participant1: generateKeyPair(),
        participant2: generateKeyPair(),
        collateral: Math.floor(Math.random() * 1000000) + 100000
    };
}

// Generate multiple test channels
function generateMultipleChannels(count = 5) {
    const channels = [];
    for (let i = 0; i < count; i++) {
        channels.push(generateTestChannel());
    }
    return channels;
}

// Generate test order data
function generateTestOrder(listingId) {
    return {
        listing_id: listingId,
        buyer_pubkey: generateKeyPair()
    };
}

// Random order status
function getRandomOrderStatus() {
    const statuses = ['pending', 'confirmed', 'shipped', 'delivered', 'completed'];
    return statuses[Math.floor(Math.random() * statuses.length)];
}

// Generate stress test data
function generateStressTestData() {
    return {
        channels: generateMultipleChannels(20),
        listings: generateMultipleListings(50),
        keypairs: Array.from({ length: 10 }, () => generateKeyPair())
    };
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        generateKeyPair,
        generateTestListing,
        generateMultipleListings,
        generateTestChannel,
        generateMultipleChannels,
        generateTestOrder,
        getRandomOrderStatus,
        generateStressTestData
    };
}
