# metrics_server
[![CI](https://github.com/loshz/metrics_server/actions/workflows/ci.yml/badge.svg)](https://github.com/loshz/metrics_server/actions/workflows/ci.yml)
[![Version](https://img.shields.io/crates/v/metrics_server.svg)](https://crates.io/crates/metrics_server)
[![Docs](https://docs.rs/metrics_server/badge.svg)](https://docs.rs/metrics_server)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/loshz/metrics_server/blob/main/LICENSE)

>**Note**: The lib's API might change before v1.0, it should **NOT** currently be considered stable.

A hassle-free, single-responsibility, safe HTTP/S server used to easily expose metrics in an application.

This crate provides a thread safe, minimalstic HTTP/S server used to buffer metrics and serve them via a standard `/metrics` endpoint. It's aim is to remove the boilerplate needed to create such simple mechanisms. It is currently somewhat oppinionated and naive in order to maintain little complexity.


## Usage

Include the lib in your `Cargo.toml` dependencies:
```toml
[dependencies]
metrics_server = "0.6"
```

### HTTP
```rust
use metrics_server::MetricsServer;

// Create a new HTTP server and start listening for requests in the background.
let server = MetricsServer::http("localhost:8001");

// Publish you application metrics.
let bytes = server.update(Vec::from([1, 2, 3, 4]));
assert_eq!(4, bytes);
```

### HTTPS
Note: there is currently no option to skip TLS cert verification.
```rust
use metrics_server::MetricsServer;

// Load TLS config.
let cert = include_bytes!("/path/to/cert.pem").to_vec();
let key = include_bytes!("/path/to/key.pem").to_vec();

// Create a new HTTPS server and start listening for requests in the background.
let server = MetricsServer::https("localhost:8443", cert, key);

// Publish you application metrics.
let bytes = server.update(Vec::from([1, 2, 3, 4]));
assert_eq!(4, bytes);
```

For more comprehensive usage, see the included [examples](./examples).
