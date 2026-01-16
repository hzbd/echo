use axum::{
    body::Bytes,
    http::{HeaderMap, Method, StatusCode, Uri}, // Import Method and Uri
    Router,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::net::SocketAddr;

// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

// Define the secret key (Corresponds to Python's b'sk_prod_123456')
const SECRET: &[u8] = b"sk_prod_123456";

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Build router
    // Use 'fallback' to catch all undefined routes (implements "accept all paths")
    let app = Router::new().fallback(webhook_handler);

    // Listen on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Universal Webhook Receiver listening on {}", addr);
    println!("   Accepting ALL paths and methods...");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Added method and uri parameters to print request details
async fn webhook_handler(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    println!("\n========================================================");
    // Print specific request Method and URI
    println!("📢 Request: {} {}", method, uri);
    println!("========================================================");

    // 1. Print all Headers
    println!("[Headers]:");
    for (name, value) in headers.iter() {
        println!("  {}: {:?}", name, value);
    }

    // 2. Print Body content
    println!("\n[Body]:");
    // Simple check to avoid flooding the console with binary data or large payloads
    if body.is_empty() {
        println!("  <Empty Body>");
    } else {
        match String::from_utf8(body.to_vec()) {
            Ok(text) => println!("{}", text),
            Err(_) => println!("  <Binary Data: {} bytes>", body.len()),
        }
    }

    println!("\n[Verification]:");
    // 3. Verification Logic (Only executes if the signature header exists)
    if let Some(signature_header) = headers.get("X-Super-Signature") {
        let signature_str = match signature_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                println!("  ❌ Error: Signature header contains invalid ASCII.");
                return StatusCode::BAD_REQUEST;
            }
        };

        // Format expectation: sha256=xxxx...
        let parts: Vec<&str> = signature_str.split('=').collect();
        if parts.len() < 2 {
            println!("  ❌ Error: Invalid signature format. Expected 'key=value'");
            return StatusCode::BAD_REQUEST;
        }
        let provided_signature = parts[1];

        // Calculate expected HMAC
        let mut mac = HmacSha256::new_from_slice(SECRET).expect("HMAC error");
        mac.update(&body);
        let expected_signature = hex::encode(mac.finalize().into_bytes());

        println!("  Secrets:    {}", String::from_utf8_lossy(SECRET));
        println!("  Provided:   {}", provided_signature);
        println!("  Calculated: {}", expected_signature);

        if provided_signature != expected_signature {
            println!("  ⛔ Result:  Invalid signature! (401)");
            return StatusCode::UNAUTHORIZED;
        } else {
            println!("  ✅ Result:  Signature verified.");
        }
    } else {
        println!("  ⚠️ Info: No 'X-Super-Signature' header. Skipping verification.");
    }

    println!("--------------------------------------------------------\n");
    StatusCode::OK
}
