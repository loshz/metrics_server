use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use http::Uri;
use log::{debug, error};
use tiny_http::{ConfigListenAddr, Method, Response, Server};

use crate::error::ServerError;

/// The default metrics URL path of the server.
pub const DEFAULT_METRICS_PATH: &str = "/metrics";

/// A thread-safe datastore for serving metrics via a HTTP/S server.
pub struct MetricsServer {
    shared: Arc<MetricsServerShared>,
    thread: Option<thread::JoinHandle<()>>,
}

struct MetricsServerShared {
    data: Mutex<Vec<u8>>,
    server: Server,
    stop: AtomicBool,
}

impl MetricsServer {
    /// Creates an empty `MetricsServer` with a configured HTTP/S server.
    pub fn new<A>(
        addr: A,
        certificate: Option<Vec<u8>>,
        private_key: Option<Vec<u8>>,
    ) -> Result<Self, ServerError>
    where
        A: ToSocketAddrs,
    {
        // Construct listener from address.
        let listener = ConfigListenAddr::from_socket_addrs(addr)
            .map_err(|e| ServerError::Create(e.to_string()))?;

        // Parse TLS config.
        let config = match (certificate, private_key) {
            #[cfg(feature = "tls")]
            (Some(certificate), Some(private_key)) => tiny_http::ServerConfig {
                addr: listener,
                ssl: Some(tiny_http::SslConfig {
                    certificate,
                    private_key,
                }),
            },
            // Default to no TLS.
            _ => tiny_http::ServerConfig {
                addr: listener,
                ssl: None,
            },
        };

        // Attempt to create a new server.
        let server = Server::new(config).map_err(|e| ServerError::Create(e.to_string()))?;

        // Create an Arc of the shared data.
        let shared = Arc::new(MetricsServerShared {
            data: Mutex::new(Vec::new()),
            server,
            stop: AtomicBool::new(false),
        });

        Ok(MetricsServer {
            shared,
            thread: None,
        })
    }

    /// Shortcut for creating an empty `MetricsServer` and starting a HTTP server on a new thread at the given address.
    ///
    /// The server will only respond synchronously as it blocks until receiving new requests.
    ///
    /// # Panics
    ///
    /// Panics if given an invalid address.
    pub fn http<A>(addr: A) -> Self
    where
        A: ToSocketAddrs,
    {
        let mut server = MetricsServer::new(addr, None, None).unwrap();
        server.serve();
        server
    }

    /// Shortcut for creating an empty `MetricsServer` and starting a HTTPS server on a new thread at the given address.
    ///
    /// The server will only respond synchronously as it blocks until receiving new requests.
    ///
    /// Note: there is currently no option to skip TLS cert verification.
    ///
    /// # Panics
    ///
    /// Panics if given an invalid address or incorrect TLS credentials.
    #[cfg(feature = "tls")]
    pub fn https<A>(addr: A, certificate: Vec<u8>, private_key: Vec<u8>) -> Self
    where
        A: ToSocketAddrs,
    {
        let mut server = MetricsServer::new(addr, Some(certificate), Some(private_key)).unwrap();
        server.serve();
        server
    }

    /// Safely updates the data in a `MetricsServer` and returns the number of bytes written.
    ///
    /// This method is protected by a mutex making it safe to call concurrently from multiple threads.
    pub fn update(&self, data: Vec<u8>) -> usize {
        let mut buf = self.shared.data.lock().unwrap();
        *buf = data;
        buf.as_slice().len()
    }

    /// Start serving requests to the /metrics URL path on the underlying server.
    ///
    /// The server will only respond synchronously as it blocks until receiving new requests.
    /// Suqsequent calls to this method will return a no-op and not affect the underlying server.
    pub fn serve(&mut self) {
        self.serve_uri(DEFAULT_METRICS_PATH.to_string())
    }

    /// Start serving requests to a specific URL path on the underlying server.
    ///
    /// The server will only respond synchronously as it blocks until receiving new requests.
    /// Suqsequent calls to this method will return a no-op and not affect the underlying server.
    pub fn serve_uri(&mut self, uri: String) {
        // Check if we already have a thread running.
        if let Some(thread) = &self.thread {
            if !thread.is_finished() {
                return;
            }
        }

        // Ensure URI is valid.
        let u = parse_uri(uri);

        // Invoking clone on Arc produces a new Arc instance, which points to the
        // same allocation on the heap as the source Arc, while increasing a reference count.
        let s = Arc::clone(&self.shared);

        // Handle requests in a new thread so we can process in the background.
        self.thread = Some(thread::spawn({
            move || {
                // Blocks until the next request is received.
                for req in s.server.incoming_requests() {
                    // Check to see if we should stop handling requests.
                    if s.stop.load(Ordering::Relaxed) {
                        break;
                    }

                    debug!(
                        "metrics_server: request received [url: '{}', remote_addr: '{}', http_version: '{}']",
                        req.url(),
                        req.remote_addr().map_or("N/A".to_string(), |v| v.to_string()),
                        req.http_version(),
                    );

                    // Only serve the specified uri path.
                    if req.url().to_lowercase() != u {
                        let res = Response::empty(404);
                        if let Err(e) = req.respond(res) {
                            error!("metrics_server error: {}", e);
                        };
                        continue;
                    }

                    // Only respond to GET requests.
                    if req.method() != &Method::Get {
                        let res = Response::empty(405);
                        if let Err(e) = req.respond(res) {
                            error!("metrics_server error: {}", e);
                        };
                        continue;
                    }

                    // Write the metrics to the response buffer.
                    let metrics = s.data.lock().unwrap();
                    let res = Response::from_data(metrics.as_slice());
                    if let Err(e) = req.respond(res) {
                        error!("metrics_server error: {}", e);
                    };
                }
            }
        }));
    }

    /// Stop serving requests and free thread resources.
    pub fn stop(mut self) -> Result<(), ServerError> {
        // Signal that we should stop handling requests and unblock the server.
        self.shared.stop.store(true, Ordering::Relaxed);
        self.shared.server.unblock();

        // Because join takes ownership of the thread, we need to call the take method
        // on the Option to move the value out of the Some variant and leave a None
        // variant in its place.
        match self.thread.take() {
            Some(thread) => thread.join().map_err(|e| {
                let err = match e.downcast_ref::<String>() {
                    Some(s) => s,
                    None => "unknown",
                };

                ServerError::Stop(err.to_string())
            }),
            None => Ok(()),
        }
    }
}

/// Validate the provided URI or return the default /metrics on error.
fn parse_uri(mut uri: String) -> String {
    if !uri.starts_with('/') {
        uri = format!("/{}", uri);
    }

    let u = match Uri::from_str(&uri) {
        Ok(u) => u.path().to_string(),
        Err(_) => {
            error!("invalid uri, defaulting to {}", DEFAULT_METRICS_PATH);
            return DEFAULT_METRICS_PATH.to_string();
        }
    };

    u.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uri() {
        let expected_default = DEFAULT_METRICS_PATH.to_string();
        let expected_valid = "/v1/metrics".to_string();

        // Invalid.
        assert_eq!(parse_uri("Hello, World!".to_string()), expected_default);
        // No slash prefix.
        assert_eq!(parse_uri("metrics".to_string()), expected_default);
        // Leading slash prefix.
        assert_eq!(parse_uri("/metrics".to_string()), expected_default);
        // Whitespace.
        assert_eq!(parse_uri(" metr ics  ".to_string()), expected_default);
        // Uppercase.
        assert_eq!(parse_uri("METRICS".to_string()), expected_default);
        // Valid.
        assert_eq!(parse_uri("/v1/metrics".to_string()), expected_valid);
    }
}
