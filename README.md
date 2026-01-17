# Echo Receiver

A lightweight, Rust-based HTTP server designed to debug and verify webhook requests. It accepts traffic on **any path** and **any HTTP method**, logs structured details to the console, and performs HMAC-SHA256 signature verification if a specific header is present.

## Features

- **Catch-All Routing**: Accepts requests on any URI (e.g., `/`, `/api/callback`, `/hooks/v1`).
- **High-Readability Logging**:
  - **Aligned Output**: Key-Value pairs are vertically aligned for easy scanning.
  - **Timestamped**: Records the exact arrival time of every request.
  - **JSON Pretty-Printing**: Automatically detects and formats JSON payloads.
- **HMAC-SHA256 Verification**: Verifies payload integrity via `X-Super-Signature` header against a secret key.
- **Flexible Configuration**: Configurable secret key and listening port via command-line arguments.
- **High Performance**: Built on [Axum](https://github.com/tokio-rs/axum) & [Tokio](https://tokio.rs/).

## Prerequisites

- Rust (latest stable version)
- Cargo

## Installation & Run

### 1. Basic Run (Default Config)
Starts server with **default secret**: `sk_prod_123456` and **default port**: `3000`.

```bash
cargo run
```

### 2. Custom Configuration
Customize the secret key and port using command-line flags:

```bash
# Custom secret + custom port
cargo run -- --secret "my_secure_secret" --port 8080

# Only custom secret (port defaults to 3000)
cargo run -- --secret "another_secret"
```

### 3. Startup Confirmation
You will see a banner confirming the active configuration:

```text
 WEBHOOK RECEIVER ONLINE
  Secret Key         : sk_prod_123456
  Listen Port        : 3000
```

## Signature Verification Rule

> **Core Logic**: The HMAC-SHA256 signature is calculated using the **raw request body bytes**.
> **Header Format**: `X-Super-Signature: sha256=hex_encoded_signature`

### How To Calculate Valid Signature
Use this one-line command to generate a valid signature for testing:

```bash
# Format: echo -n "BODY" | openssl dgst -sha256 -hmac "SECRET"
echo -n "hello world" | openssl dgst -sha256 -hmac "sk_prod_123456"
```
*Result: `e6fa8032599cfbb055e4835c5daa906a1758125d56134f50b2a0af74150c8959`*

## Usage Examples

### 1. Request Without Signature (Logging Only)
Any path is accepted. Verification is skipped if the header is missing.

```bash
curl -X POST http://127.0.0.1:3000/my/callback \
     -H "Content-Type: application/json" \
     -d '{"status":"ok"}'
```

### 2. Request With VALID Signature
Sends a JSON body. The server will pretty-print the JSON and verify the signature.

**Secret**: `sk_prod_123456`
**Body**: `{"order_id":12345,"status":"paid","items":["apple","banana"]}`
**Signature**: `f52e59779307d727276326e0300a7206b020023a1a364239276d497c276f573d`

```bash
curl -X POST http://127.0.0.1:3000/webhook \
  -H "Content-Type: application/json" \
  -H "X-Super-Signature: sha256=f52e59779307d727276326e0300a7206b020023a1a364239276d497c276f573d" \
  -d '{"order_id":12345,"status":"paid","items":["apple","banana"]}'
```

### 3. Request With INVALID Signature
The server calculates the hash using the active secret and compares it.

```bash
curl -X POST http://127.0.0.1:3000/webhook \
     -H "X-Super-Signature: sha256=bad_signature_123" \
     -d "hello world"
```

## Sample Console Log

The server produces clean, structured logs for every request:

```text
==================================================
  Time               : 2023-10-27 10:30:45
  Method             : POST
  Path               : /webhook

 HEADERS
  host               : 127.0.0.1:3000
  content-type       : application/json
  x-super-signature  : sha256=f52e5...

 PAYLOAD
  Body               : {
                         "items": [
                           "apple",
                           "banana"
                         ],
                         "order_id": 12345,
                         "status": "paid"
                       }

 SIGNATURE VERIFICATION
  Secret Used        : sk_prod_123456
  Expected           : f52e5...
  Received           : f52e5...
  Result             : PASS
==================================================
```

## License

MIT
