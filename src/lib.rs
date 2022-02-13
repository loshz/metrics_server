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
//! let server = MetricsServer::new();
//! server.serve("localhost:8001");
//! server.update(vec!([1, 2, 3, 4]));
//! ```
mod server;

pub use server::MetricsServer;
