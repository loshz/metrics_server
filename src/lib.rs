#![warn(missing_docs, rustdoc::missing_doc_code_examples)]

//! A hassle-free, single-responsibility HTTP server used to easily expose metrics in an application.
//!
//! This crate provides a thread safe, minimalstic HTTP server used to buffer metrics and serve
//! them via a standard /metrics endpoint. It's aim is to remove the boilerplate needed to
//! create such simple mechanisms. It is currently somewhat oppinionated and naive in order to
//! maintain little complexity.
//!
//! # Examples
//!
//! ```
//! use metrics_server::MetricsServer;
//!
//! // Create a new server and start it in the background.
//! let server = MetricsServer::new("localhost:8001");
//!
//! // Publish you application metrics periodically.
//! let bytes = server.update(Vec::from([1, 2, 3, 4]));
//! assert_eq!(4, bytes);
//! ```
mod server;

pub use server::MetricsServer;
