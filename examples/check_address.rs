use tari_l2_marketplace::Wallet;

fn main() {
    // Import the wallet using the private key
    let private_key = "260324a664227eeec80169306a9ded3017b028fcaf3cfb36f3a3350f11d5cc38";

    println!("Importing wallet from private key...");
    let wallet = Wallet::from_private_key(private_key).expect("Failed to import wallet");

    println!("\n=== WALLET ADDRESSES ===");
    println!("Emoji Address: {}", wallet.address());
    println!("Emoji count: {}", wallet.address().chars().count());
    println!("\nHex Address: {}", wallet.address_hex());
    println!("Hex length: {}", wallet.address_hex().len());
    println!("\nPublic Key (raw): {}", wallet.public_key_hex());
    println!("Public key length: {}", wallet.public_key_hex().len());

    println!("\n=== FOR MINING ===");
    println!("Use the HEX ADDRESS:");
    println!("{}", wallet.address_hex());
}
