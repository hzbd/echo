# Echo Receiver
A lightweight, Rust-based HTTP server designed to debug and verify webhook requests. It accepts traffic on **any path** and **any HTTP method**, logs the details to the console, and performs HMAC-SHA256 signature verification if a specific header is present.

## Features
- **Catch-All Routing**: Accepts requests on any URI (e.g., `/`, `/api/callback`, `/hooks/v1`, `/webhook`).
- **Full Request Logging**: Prints HTTP Method, Request URI, all Headers and Body content (plain text / binary size) to console for debugging.
- **HMAC-SHA256 Signature Verification**: Automatically verify payload integrity via `X-Super-Signature` header.
- **Flexible Configuration**: Configurable secret key and listening port via command-line arguments (no hardcoding).
- **High Performance**: Built on [Axum](https://github.com/tokio-rs/axum) & [Tokio](https://tokio.rs/) with async runtime, zero redundant memory copy.
- **Standard HTTP Responses**: Returns `200 OK`/`400 Bad Request`/`401 Unauthorized` based on verification result.

## Prerequisites
- Rust (latest stable version)
- Cargo (Rust's build tool)

## Installation & Run
### Basic Run (Default Config)
Starts server with **default secret**: `sk_prod_123456` and **default port**: `3000`
```bash
cargo run
```
Server listens on `0.0.0.0:3000`

### Custom Configuration (Command-line Arguments)
Supports custom secret key and port, use `--secret` / `--port` flags:
```bash
# Custom secret + custom port
cargo run -- --secret "your_custom_secret" --port 8080

# Only custom secret (keep port 3000)
cargo run -- --secret "sk_prod_888888"

# Only custom port (keep default secret)
cargo run -- --port 9090
```

### Startup Confirmation
Server prints active config on launch for verification:
```
--------------------------------------------------------
Active Secret:   'sk_prod_123456'
Listening Port:  3000
--------------------------------------------------------
Server started on 0.0.0.0:3000
```

## Signature Verification Rule
> Core Logic: HMAC-SHA256 signature is calculated using **raw request body only** (no extra line breaks/characters appended).
> Header Format: `X-Super-Signature: sha256=hex_encoded_signature`

### How To Calculate Valid Signature
Use this **one-line OpenSSL command** (terminal) to generate a valid signature for any payload/secret combination (matches server logic exactly):
```bash
# Universal formula
echo -n "your_request_body" | openssl dgst -sha256 -hmac "your_secret_key"
```
The output hex string = valid signature for your request.

## Usage Examples
### 1. Request Without Signature (Logging Only)
All paths/methods work, server logs details and returns `200 OK` (skip verification):
```bash
curl -X POST http://127.0.0.1:3000/any/custom/path \
     -H "Content-Type: application/json" \
     -d '{"event":"success","data":{"id":123}}'
```

### 2. Request With VALID Signature (Success, 200 OK)
**Config**: Secret = `sk_prod_123456`, Body = `hello world`
**Valid Signature**: `e6fa8032599cfbb055e4835c5daa906a1758125d56134f50b2a0af74150c8959`
```bash
curl -X POST http://127.0.0.1:3000/webhook \
     -H "X-Super-Signature: sha256=e6fa8032599cfbb055e4835c5daa906a1758125d56134f50b2a0af74150c8959" \
     -d "hello world"
```
**Result**: Signature verified successfully.

### 3. Request With INVALID Signature (Failed, 401 Unauthorized)
```bash
curl -X POST http://127.0.0.1:3000/webhook \
     -H "X-Super-Signature: sha256=wrong_signature_123456" \
     -d "hello world"
```

### 4. Malformed Signature Header (Invalid Format, 400 Bad Request)
```bash
curl -X POST http://127.0.0.1:3000/webhook \
     -H "X-Super-Signature: missing_sha256_prefix" \
     -d "hello world"
```

## Sample Console Log
Complete request details logged for every incoming traffic:
```
========================================================
📢 Request: POST /webhook
========================================================
[Headers]:
  host: "127.0.0.1:3000"
  user-agent: "curl/8.7.1"
  accept: "*/*"
  x-super-signature: "sha256=e6fa8032599cfbb055e4835c5daa906a1758125d56134f50b2a0af74150c8959"
  content-length: "11"
  content-type: "application/x-www-form-urlencoded"

[Body]:
hello world

[Verification]:
  Secrets:    'sk_prod_123456'
  Provided:   e6fa8032599cfbb055e4835c5daa906a1758125d56134f50b2a0af74150c8959
  Calculated: e6fa8032599cfbb055e4835c5daa906a1758125d56134f50b2a0af74150c8959
  Result:  Signature verified successfully.
--------------------------------------------------------
```

## License
MIT
