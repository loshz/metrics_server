# Metrics Server

[![CI](https://github.com/syscll/metrics-server/actions/workflows/ci.yml/badge.svg)](https://github.com/syscll/metrics-server/actions/workflows/ci.yml)
[![Version](https://img.shields.io/crates/v/metrics-server.svg)](https://crates.io/crates/metrics-server)
[![Docs](https://docs.rs/metrics-server/badge.svg)](https://docs.rs/metrics-server)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/syscll/metrics-server/blob/main/LICENSE)

A hassle-free, single-responsibility, safe HTTP server used to easily expose metrics in your application.

_**Note: this library is NOT production ready! Use with caution and submit bugs where possible.**_

## Usage

Include the lib in your `Cargo.toml` dependencies:
```toml
[dependencies]
metrics_server = "0.1"
```

In your application:
```rust
use metrics_server::MetricsServer;

// Create a new server and start it in the background.
let server = MetricsServer::new();
server.serve("localhost:8001");

// Publish you application metrics periodically.
let bytes = server.update(Vec::from([1, 2, 3, 4]));
assert_eq!(bytes, 4);
```

## TOOD
- [ ] Add integration tests.
- [ ] Add Prometheus examples.
