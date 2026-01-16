use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    Router,
};
use clap::Parser;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::net::SocketAddr;
use std::sync::Arc;

// Type alias for HMAC-SHA256 encryption algorithm
type HmacSha256 = Hmac<Sha256>;

// Command line arguments structure definition
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Verification secret key (default value: sk_prod_123456)
    #[arg(short, long, default_value = "sk_prod_123456")]
    secret: String,

    /// Server listening port (default value: 3000)
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

// Application global state for concurrent safe sharing
#[derive(Clone)]
struct AppState {
    secret: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for logging functionality
    tracing_subscriber::fmt().init();

    // Parse command line arguments
    let args = Args::parse();

    // Print current runtime configuration information
    println!("--------------------------------------------------------");
    println!("Active Secret:   '{}'", args.secret);
    println!("Listening Port:  {}", args.port);
    println!("--------------------------------------------------------");

    // Wrap state with Arc for thread-safe sharing across axum handlers
    let state = Arc::new(AppState {
        secret: args.secret.clone(),
    });

    // Build axum router with global state injection & fallback handler
    let app = Router::new()
        .fallback(webhook_handler)
        .with_state(state);

    // Bind server to ipv4 0.0.0.0 with specified port
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Server started on {}", addr);

    // Start TCP listener and axum HTTP server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Unified webhook request handler with HMAC signature verification logic
async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    println!("\n========================================================");
    println!("Request: {} {}", method, uri);
    println!("========================================================");

    // Print all incoming request headers
    println!("[Headers]:");
    for (name, value) in headers.iter() {
        println!("  {}: {:?}", name, value);
    }

    // Print request body content with UTF8 check
    println!("\n[Body]:");
    if body.is_empty() {
        println!("  <Empty Body>");
    } else {
        match String::from_utf8(body.to_vec()) {
            Ok(text) => println!("{}", text),
            Err(_) => println!("  <Binary Data: {} bytes>", body.len()),
        }
    }

    // Start HMAC signature verification process
    println!("\n[Verification]:");
    if let Some(signature_header) = headers.get("X-Super-Signature") {
        // Parse signature header value to UTF8 string
        let signature_str = match signature_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                println!("Error: Invalid signature header encoding.");
                return StatusCode::BAD_REQUEST;
            }
        };

        // Split signature format: sha256=signature_value
        let parts: Vec<&str> = signature_str.split('=').collect();
        if parts.len() < 2 {
            println!("Error: Malformed signature header format.");
            return StatusCode::BAD_REQUEST;
        }
        let provided_signature = parts[1];

        // Initialize HMAC-SHA256 instance with application secret key
        let mut mac = HmacSha256::new_from_slice(state.secret.as_bytes())
            .expect("HMAC initialization failed: invalid secret key bytes");

        // Update HMAC context with raw request body only (core verification rule)
        mac.update(&body);
        // mac.update(b"\n"); // Optional: append line break if required by webhook provider

        // Generate expected signature & encode to hex string
        let expected_signature = hex::encode(mac.finalize().into_bytes());

        // Print debug info for signature verification
        println!("  Secrets:    '{}'", state.secret);
        println!("  Provided:   {}", provided_signature);
        println!("  Calculated: {}", expected_signature);

        // Signature comparison & response
        if provided_signature != expected_signature {
            println!("Result:  Invalid signature! (401 Unauthorized)");
            return StatusCode::UNAUTHORIZED;
        } else {
            println!("Result:  Signature verified successfully.");
        }
    } else {
        println!("Info: X-Super-Signature header is missing, skip verification.");
    }

    println!("--------------------------------------------------------\n");
    StatusCode::OK
}
