use std::net::ToSocketAddrs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

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
    /// Creates an empty `MetricsServer` and starts a HTTP server on a new thread at the given address.
    ///
    /// This server will only respond synchronously as it blocks until receiving new requests.
    ///
    /// # Panics
    ///
    /// Panics if given an invalid address.
    pub fn new<A>(addr: A) -> Self
    where
        A: ToSocketAddrs,
    {
        let config = tiny_http::ServerConfig { addr, ssl: None };

        MetricsServer::serve(config)
    }

    /// Creates an empty `MetricsServer` and starts a HTTPS server on a new thread at the given address.
    ///
    /// This server will only respond synchronously as it blocks until receiving new requests.
    ///
    /// Note: there is currently no option to skip TLS cert verification.
    ///
    /// # Panics
    ///
    /// Panics if given an invalid address or incorrect TLS credentials.
    pub fn https<A>(addr: A, certificate: Vec<u8>, private_key: Vec<u8>) -> Self
    where
        A: ToSocketAddrs,
    {
        let config = tiny_http::ServerConfig {
            addr,
            ssl: Some(tiny_http::SslConfig {
                certificate,
                private_key,
            }),
        };

        MetricsServer::serve(config)
    }

    /// Safely updates the data in a `MetricsServer` and returns the number of bytes written.
    ///
    /// This method is protected by a mutex making it safe
    /// to call concurrently from multiple threads.
    pub fn update(&self, data: Vec<u8>) -> usize {
        let mut buf = self.shared.data.lock().unwrap();
        *buf = data;
        buf.as_slice().len()
    }

    fn serve<A>(config: tiny_http::ServerConfig<A>) -> Self
    where
        A: ToSocketAddrs,
    {
        // Create an Arc of the shared data.
        let shared = Arc::new(MetricsServerShared {
            data: Mutex::new(Vec::new()),
            server: Server::new(config).unwrap(),
            stop: AtomicBool::new(false),
        });

        // Invoking clone on Arc produces a new Arc instance, which points to the
        // same allocation on the heap as the source Arc, while increasing a reference count.
        let s = Arc::clone(&shared);

        // Handle requests in a new thread so we can process in the background.
        let thread = Some(thread::spawn({
            move || {
                // Blocks until the next request is received.
                for req in s.server.incoming_requests() {
                    // Check to see if we should stop handling requests.
                    if s.stop.load(Ordering::Relaxed) {
                        break;
                    }

                    // Only serve the /metrics path.
                    if req.url() != "/metrics" {
                        let res = Response::empty(404);
                        let _ = req.respond(res);
                        continue;
                    }

                    // Only respond to GET requests.
                    if req.method() != &Method::Get {
                        let res = Response::empty(405);
                        let _ = req.respond(res);
                        continue;
                    }

                    // Write the metrics to the response buffer.
                    let metrics = s.data.lock().unwrap();
                    let res = Response::from_data(metrics.as_slice());
                    if let Err(e) = req.respond(res) {
                        eprintln!("metrics_server error: {}", e);
                    };
                }
            }
        }));

        MetricsServer { shared, thread }
    }
}

impl Drop for MetricsServer {
    // TODO: should I really be doing this inside drop? It _could_ panic,
    // so maybe a shutdown method would be better?
    fn drop(&mut self) {
        // Signal that we should stop handling requests and unblock the server.
        self.shared.stop.store(true, Ordering::Relaxed);
        self.shared.server.unblock();

        // Because join takes ownership of the thread, we need call the take method
        // on the Option to move the value out of the Some variant and leave a None
        // variant in its place.
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
