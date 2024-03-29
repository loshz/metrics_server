#![warn(missing_docs, rustdoc::missing_doc_code_examples)]

//! A hassle-free, single-responsibility HTTP/S server used to easily expose metrics in an application.
//!
//! This crate provides a thread safe, minimalstic HTTP/S server used to buffer metrics and serve
//! them via a standard `/metrics` endpoint. It's aim is to remove the boilerplate needed to
//! create such simple mechanisms. It is currently somewhat oppinionated and naive in order to
//! maintain little complexity.
//!
//! # Examples
//!
//! ## Start a HTTP server:
//!
//! ```rust
//! use metrics_server::MetricsServer;
//!
//! // Create a new HTTP server and start listening for requests in the background.
//! let server = MetricsServer::http("localhost:8001");
//!
//! // Publish your application metrics.
//! let bytes = server.update("my_awesome_metric = 10".into());
//! assert_eq!(22, bytes);
//!
//! // Stop the server.
//! server.stop().unwrap();
//! ```
//!
//! ## Start a HTTPS server:
//!
//! ```rust
//! use metrics_server::MetricsServer;
//!
//! // Load TLS config.
//! let cert = include_bytes!("/path/to/cert.pem").to_vec();
//! let key = include_bytes!("/path/to/key.pem").to_vec();
//!
//! // Create a new HTTPS server and start listening for requests in the background.
//! let server = MetricsServer::https("localhost:8443", cert, key);
//!
//! // Publish your application metrics.
//! let bytes = server.update("my_awesome_metric = 10".into());
//! assert_eq!(22, bytes);
//!
//! // Stop the server.
//! server.stop().unwrap();
//! ```
//!
//! ## Serve a custom URL
//!
//! ```rust
//! use metrics_server::MetricsServer;
//!
//! // Create a new server and specify the URL path to serve.
//! let mut server = MetricsServer::new("localhost:8001", None, None);
//! server.serve_uri("/path/to/metrics");
//!
//! // Publish your application metrics.
//! let bytes = server.update("my_awesome_metric = 10".into());
//! assert_eq!(22, bytes);
//!
//! // Stop the server.
//! server.stop().unwrap();
//! ```
mod error;
mod server;

pub use error::ServerError;
pub use server::{MetricsServer, DEFAULT_METRICS_PATH};
