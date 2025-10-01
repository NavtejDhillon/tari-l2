use std::io::{self, Write};
use serde_json::{json, Value};
use std::net::TcpStream;
use std::io::{BufRead, BufReader};

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║   Tari L2 Marketplace CLI Client            ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();

    loop {
        println!("\n📋 MAIN MENU:");
        println!("  1. List all channels");
        println!("  2. Get channel info");
        println!("  3. Get balance");
        println!("  4. Create listing (stub)");
        println!("  5. Create order (stub)");
        println!("  6. Transfer funds (stub)");
        println!("  7. Switch node");
        println!("  8. Exit");
        print!("\nSelect option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => list_channels(),
            "2" => get_channel_info(),
            "3" => get_balance(),
            "4" => println!("📝 Create listing - Coming in v0.2.0"),
            "5" => println!("🛒 Create order - Coming in v0.2.0"),
            "6" => println!("💸 Transfer - Coming in v0.2.0"),
            "7" => switch_node(),
            "8" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option"),
        }
    }
}

fn get_node_port() -> u16 {
    // Default to node 1
    18000
}

fn send_rpc_request(method: &str, params: Option<Value>) -> Result<Value, String> {
    let port = get_node_port();

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let request_str = format!("{}\n", serde_json::to_string(&request).unwrap());

    match TcpStream::connect(format!("127.0.0.1:{}", port)) {
        Ok(mut stream) => {
            use std::io::Write;
            stream.write_all(request_str.as_bytes()).map_err(|e| e.to_string())?;

            let mut reader = BufReader::new(stream);
            let mut response = String::new();
            reader.read_line(&mut response).map_err(|e| e.to_string())?;

            let json: Value = serde_json::from_str(&response).map_err(|e| e.to_string())?;

            if let Some(error) = json.get("error") {
                if !error.is_null() {
                    return Err(format!("RPC Error: {}", error));
                }
            }
            Ok(json["result"].clone())
        }
        Err(e) => Err(format!("Connection failed: {}. Is the node running?", e))
    }
}

fn list_channels() {
    println!("\n📊 Listing all channels...");

    match send_rpc_request("list_channels", None) {
        Ok(result) => {
            if result.is_array() {
                let channels = result.as_array().unwrap();
                if channels.is_empty() {
                    println!("   No channels found.");
                    println!("   💡 Tip: Run the demo to create test channels:");
                    println!("      cargo run --example marketplace_demo");
                } else {
                    println!("   Found {} channel(s):", channels.len());
                    for (i, channel) in channels.iter().enumerate() {
                        println!("   {}. {}", i + 1, serde_json::to_string_pretty(channel).unwrap());
                    }
                }
            } else {
                println!("   ✅ Result: {}", serde_json::to_string_pretty(&result).unwrap());
            }
        }
        Err(e) => println!("   ❌ Error: {}", e)
    }
}

fn get_channel_info() {
    print!("\nEnter channel ID (hex): ");
    io::stdout().flush().unwrap();

    let mut channel_id = String::new();
    io::stdin().read_line(&mut channel_id).unwrap();
    let channel_id = channel_id.trim();

    if channel_id.is_empty() {
        println!("❌ Channel ID cannot be empty");
        return;
    }

    println!("\n📋 Getting channel info...");

    let params = json!({
        "channel_id": channel_id
    });

    match send_rpc_request("get_channel_info", Some(params)) {
        Ok(result) => {
            println!("   ✅ Channel Info:");
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => println!("   ❌ Error: {}", e)
    }
}

fn get_balance() {
    print!("\nEnter channel ID (hex): ");
    io::stdout().flush().unwrap();
    let mut channel_id = String::new();
    io::stdin().read_line(&mut channel_id).unwrap();

    print!("Enter participant public key (hex): ");
    io::stdout().flush().unwrap();
    let mut participant = String::new();
    io::stdin().read_line(&mut participant).unwrap();

    let channel_id = channel_id.trim();
    let participant = participant.trim();

    if channel_id.is_empty() || participant.is_empty() {
        println!("❌ Both fields are required");
        return;
    }

    println!("\n💰 Getting balance...");

    let params = json!({
        "channel_id": channel_id,
        "participant": participant
    });

    match send_rpc_request("get_balance", Some(params)) {
        Ok(result) => {
            println!("   ✅ Balance: {} units", result);
        }
        Err(e) => println!("   ❌ Error: {}", e)
    }
}

fn switch_node() {
    println!("\n🔄 Node switching coming in v0.2.0");
    println!("   Currently connected to: 127.0.0.1:{}", get_node_port());
}
