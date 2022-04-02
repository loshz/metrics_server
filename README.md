# Metrics Server
[![CI](https://github.com/loshz/metrics_server/actions/workflows/ci.yml/badge.svg)](https://github.com/loshz/metrics_server/actions/workflows/ci.yml)
[![Version](https://img.shields.io/crates/v/metrics_server.svg)](https://crates.io/crates/metrics_server)
[![Docs](https://docs.rs/metrics_server/badge.svg)](https://docs.rs/metrics_server)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/loshz/metrics_server/blob/main/LICENSE)

A hassle-free, single-responsibility, safe HTTP server used to easily expose metrics in an application.

## Usage

Include the lib in your `Cargo.toml` dependencies:
```toml
[dependencies]
metrics_server = "0.2"
```

In your application:
```rust
use metrics_server::MetricsServer;

// Create a new server and start it in the background.
let mut server = MetricsServer::new();
server.serve("localhost:8001");

// Publish you application metrics periodically.
let bytes = server.update(Vec::from([1, 2, 3, 4]));
assert_eq!(bytes, 4);
```

For more comprehensive usage, check out the included [examples](./examples).
