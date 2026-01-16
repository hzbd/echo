
# Universal Webhook Receiver

A lightweight, Rust-based HTTP server designed to debug and verify webhook requests. It accepts traffic on **any path** and **any HTTP method**, logs the details to the console, and performs HMAC-SHA256 signature verification if a specific header is present.

## Features

-   **Catch-All Routing**: Accepts requests on any URI (e.g., `/`, `/api/callback`, `/hooks/v1`).
-   **Full Logging**: Prints HTTP Method, URI, Headers, and Body content to the console.
-   **Signature Verification**: Automatically verifies the payload if the `X-Super-Signature` header is detected using HMAC-SHA256.
-   **High Performance**: Built on [Axum](https://github.com/tokio-rs/axum) and [Tokio](https://tokio.rs/).

## Prerequisites

-   Rust (latest stable version)
-   Cargo

## Installation & Run

1.  Clone the repository or navigate to the project directory.
2.  Run the server:

```bash
cargo run
```

The server will start listening on `0.0.0.0:3000`.

## Configuration

The HMAC secret key is currently hardcoded in `src/main.rs`:

```rust
const SECRET: &[u8] = b"sk_prod_123456";
```

## Usage Examples

### 1. Send a request without signature (Logging only)

Any path is accepted. The server will log the request and return `200 OK`.

```bash
curl -X POST http://127.0.0.1:3000/any/path/you/want \
     -d '{"status": "alive"}'
```

### 2. Send a request with a VALID signature

Assuming the secret is `sk_prod_123456` and the body is `hello world`.
Calculated Signature: `60037e831627993a4667d7924a357f00d0263f350c9dc03233b2e56306e92750`

```bash
curl -X POST http://127.0.0.1:3000/webhook \
     -H "X-Super-Signature: sha256=60037e831627993a4667d7924a357f00d0263f350c9dc03233b2e56306e92750" \
     -d "hello world"
```

**Result:** Server logs "✅ Result: Signature verified." and returns `200 OK`.

### 3. Send a request with an INVALID signature

```bash
curl -X POST http://127.0.0.1:3000/webhook \
     -H "X-Super-Signature: sha256=invalid_signature" \
     -d "hello world"
```

**Result:** Server logs "⛔ Result: Invalid signature!" and returns `401 Unauthorized`.

## Dependencies

-   `axum`: Web framework.
-   `tokio`: Async runtime.
-   `hmac` & `sha2`: Cryptography.
-   `tracing`: Logging utilities.
