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
use chrono::Local;

// Type alias for HMAC-SHA256 encryption algorithm
type HmacSha256 = Hmac<Sha256>;

// --- ANSI Color Constants ---
const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const GRAY: &str = "\x1b[90m";

// Formatting Constants
const KEY_WIDTH: usize = 18;

// Command line arguments structure definition
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Verification secret key (default: sk_prod_123456)
    #[arg(short, long, default_value = "sk_prod_123456")]
    secret: String,

    /// Server listening port (default: 3000)
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
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Parse command line arguments
    let args = Args::parse();

    // Helper macro for aligned printing on startup
    let print_startup = |key: &str, val: String| {
        println!("  {:<w$} : {}{}{}", key, YELLOW, val, RESET, w = KEY_WIDTH);
    };

    println!("");
    println!(" WEBHOOK RECEIVER ONLINE");
    print_startup("Secret Key", args.secret.clone());
    print_startup("Listen Port", args.port.to_string());
    println!("");

    // Wrap state with Arc for thread-safe sharing
    let state = Arc::new(AppState {
        secret: args.secret.clone(),
    });

    // Build axum router with fallback to accept any path
    let app = Router::new()
        .fallback(webhook_handler)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    // Start TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Unified webhook request handler
async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // Capture current timestamp
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // --- Helper 1: Print Key-Value (Single Line) ---
    // Output format: "  KEY_NAME           : VALUE"
    let log_kv = |key: &str, val: &str, color: &str| {
        println!(
            "  {}{:<w$}{} : {}{}{}",
            GRAY, key, RESET,
            color, val, RESET,
            w = KEY_WIDTH
        );
    };

    // --- Helper 2: Print Multi-line Block (Aligned) ---
    // Used for printing JSON body neatly with indentation
    let log_multiline = |key: &str, text: &str, color: &str| {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            log_kv(key, "<Empty>", GRAY);
            return;
        }

        // Print first line normally with the Key
        log_kv(key, lines[0], color);

        // Print subsequent lines with padding to align with the value column
        // Padding = 2 spaces + KEY_WIDTH + " : " (3 chars)
        let padding = " ".repeat(2 + KEY_WIDTH + 3);

        for line in lines.iter().skip(1) {
            println!("{}{}{}{}", padding, color, line, RESET);
        }
    };

    println!("==================================================");

    // 1. Basic Request Info
    log_kv("Time", &timestamp, BLUE);
    log_kv("Method", method.as_str(), BLUE);
    log_kv("Path", uri.path(), BLUE);
    println!("");

    // 2. Headers
    println!(" HEADERS");
    for (name, value) in headers.iter() {
        let val_str = value.to_str().unwrap_or("<binary>");
        // Truncate very long headers to keep layout clean
        let display_val = if val_str.len() > 50 {
            format!("{}...", &val_str[..47])
        } else {
            val_str.to_string()
        };
        log_kv(name.as_str(), &display_val, RESET);
    }
    println!("");

    // 3. Payload (With Auto JSON Formatting)
    println!(" PAYLOAD");
    if body.is_empty() {
        log_kv("Body", "<Empty>", GRAY);
    } else {
        // Try to parse the raw body bytes as JSON
        let display_text = match serde_json::from_slice::<serde_json::Value>(&body) {
            // If valid JSON, verify if we can pretty-print it
            Ok(json_val) => serde_json::to_string_pretty(&json_val)
                .unwrap_or_else(|_| String::from_utf8_lossy(&body).to_string()),
            // If not JSON, print as raw string
            Err(_) => String::from_utf8_lossy(&body).to_string(),
        };

        log_multiline("Body", &display_text, RESET);
    }
    println!("");

    // 4. Verification
    println!(" SIGNATURE VERIFICATION");

    let result_code = if let Some(signature_header) = headers.get("X-Super-Signature") {
        let signature_str = signature_header.to_str().unwrap_or("");

        // Expected format: algo=hash (e.g., sha256=abcdef...)
        if let Some((_, provided_sign)) = signature_str.split_once('=') {
            // Initialize HMAC-SHA256 with the secret from State (CLI args)
            let mut mac = HmacSha256::new_from_slice(state.secret.as_bytes())
                .expect("HMAC init failed");

            // IMPORTANT: Verify against the raw `body` bytes, NOT the pretty-printed string
            mac.update(&body);
            let expected_sign = hex::encode(mac.finalize().into_bytes());

            log_kv("Secret Used", &state.secret, GRAY);
            log_kv("Expected", &expected_sign, GRAY);
            log_kv("Received", provided_sign, GRAY);

            if provided_sign == expected_sign {
                log_kv("Result", "PASS", GREEN);
                StatusCode::OK
            } else {
                log_kv("Result", "FAIL (Mismatch)", RED);
                StatusCode::UNAUTHORIZED
            }
        } else {
            log_kv("Result", "FAIL (Format Error)", RED);
            StatusCode::BAD_REQUEST
        }
    } else {
        log_kv("Result", "SKIPPED (No Header)", YELLOW);
        StatusCode::OK
    };

    println!("==================================================\n");

    result_code
}
