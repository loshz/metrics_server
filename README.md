# metrics_server
[![CI](https://github.com/loshz/metrics_server/actions/workflows/ci.yml/badge.svg)](https://github.com/loshz/metrics_server/actions/workflows/ci.yml)
[![Version](https://img.shields.io/crates/v/metrics_server.svg)](https://crates.io/crates/metrics_server)
[![Docs](https://docs.rs/metrics_server/badge.svg)](https://docs.rs/metrics_server)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/loshz/metrics_server/blob/main/LICENSE)

>**Note**: This lib's API should _**NOT**_ currently be considered stable - it might change before v1.

A hassle-free, single-responsibility, safe HTTP/S server used to easily expose metrics in an application.

This crate provides a thread safe, minimalstic HTTP/S server used to buffer metrics and serve them via a standard `/metrics` endpoint. It's aim is to remove the boilerplate needed to create such simple mechanisms. It is currently somewhat oppinionated and naive in order to maintain little complexity.


## Usage

Include the lib in your `Cargo.toml` dependencies:
```toml
[dependencies]
metrics_server = "0.15"
```

To enable TLS support, pass the optional feature flag:
```toml
[dependencies]
metrics_server = { version = "0.15", features = ["tls"] }
```

### HTTP
```rust
use metrics_server::MetricsServer;

// Create a new HTTP server and start listening for requests in the background.
let server = MetricsServer::http("localhost:8001");

// Publish your application metrics.
let bytes = server.update("my_awesome_metric = 10".into());
assert_eq!(22, bytes);

// Stop the server.
server.stop().unwrap();
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

// Publish your application metrics.
let bytes = server.update("my_awesome_metric = 10".into());
assert_eq!(22, bytes);

// Stop the server.
server.stop().unwrap();
```

### Serve a custom URL
```rust
use metrics_server::MetricsServer;

// Create a new server and specify the URL path to serve.
let mut server = MetricsServer::new("localhost:8001", None, None);
server.serve_uri("/path/to/metrics");

// Publish your application metrics.
let bytes = server.update("my_awesome_metric = 10".into());
assert_eq!(22, bytes);

// Stop the server.
server.stop().unwrap();
```

For more comprehensive usage, see the included [examples](./examples).
