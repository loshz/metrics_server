use std::net::ToSocketAddrs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::error::ServerError;

use log::{debug, error};
use tiny_http::{Method, Response, Server};

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
        // Parse TLS config.
        let config = match (certificate, private_key) {
            #[cfg(feature = "tls")]
            (Some(certificate), Some(private_key)) => tiny_http::ServerConfig {
                addr,
                ssl: Some(tiny_http::SslConfig {
                    certificate,
                    private_key,
                }),
            },
            // Default to no TLS.
            _ => tiny_http::ServerConfig { addr, ssl: None },
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
        MetricsServer::new(addr, None, None).unwrap().serve()
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
        MetricsServer::new(addr, Some(certificate), Some(private_key))
            .unwrap()
            .serve()
    }

    /// Safely updates the data in a `MetricsServer` and returns the number of bytes written.
    ///
    /// This method is protected by a mutex making it safe to call concurrently from multiple threads.
    pub fn update(&self, data: Vec<u8>) -> usize {
        let mut buf = self.shared.data.lock().unwrap();
        *buf = data;
        buf.as_slice().len()
    }

    /// Start serving requests on the underlying server.
    ///
    /// The server will only respond synchronously as it blocks until receiving new requests.
    pub fn serve(mut self) -> Self {
        // Check if we already have a thread running.
        if let Some(thread) = &self.thread {
            if !thread.is_finished() {
                return self;
            }
        }

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
                        "metrics_server: request received [url: {}, remote addr: {}, http version: {}]",
                        req.url(),
                        req.remote_addr(),
                        req.http_version(),
                    );

                    // Only serve the /metrics path.
                    if req.url() != "/metrics" {
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

        self
    }
}

impl Drop for MetricsServer {
    // TODO: should we really be doing this inside drop? It _could_ panic,
    // so maybe a shutdown/stop method would be better?
    fn drop(&mut self) {
        // Signal that we should stop handling requests and unblock the server.
        self.shared.stop.store(true, Ordering::Relaxed);
        self.shared.server.unblock();

        // Because join takes ownership of the thread, we need to call the take method
        // on the Option to move the value out of the Some variant and leave a None
        // variant in its place.
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
