use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Method, Response, Server};

/// A thread-safe datastore for serving metrics via a HTTP server.
pub struct MetricsServer {
    fields: Arc<Fields>,
    thread: Option<thread::JoinHandle<()>>,
}

struct Fields {
    data: Mutex<Vec<u8>>,
    server: Server,
    stop: AtomicBool,
}

impl MetricsServer {
    /// Creates a new empty `MetricsServer`.
    ///
    /// Starts a simple HTTP server on a new thread at the given address and expose the stored metrics.
    /// This server is intended to only be queried synchronously as it blocks upon receiving
    /// each request.
    ///
    /// # Examples
    ///
    /// ```
    /// use metrics_server::MetricsServer;
    ///
    /// let server = MetricsServer::new("localhost:8001");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if given an invalid address.
    pub fn new(addr: &str) -> Self {
        let fields = Arc::new(Fields {
            data: Mutex::new(Vec::new()),
            server: Server::http(addr).unwrap(),
            stop: AtomicBool::new(false),
        });

        let f = Arc::clone(&fields);

        // Handle requests in a new thread so we can process in the background.
        let thread = Some(thread::spawn({
            move || {
                // Blocks until the next request is received.
                for req in f.server.incoming_requests() {
                    // Check to see if we should stop handling requests.
                    if f.stop.load(Ordering::Relaxed) {
                        break;
                    }

                    // Only respond to GET requests.
                    if req.method() != &Method::Get {
                        let res = Response::empty(405);
                        let _ = req.respond(res);
                        continue;
                    }

                    // Only serve the /metrics path.
                    if req.url() != "/metrics" {
                        let res = Response::empty(404);
                        let _ = req.respond(res);
                        continue;
                    }

                    // Write the metrics to the response buffer.
                    let metrics = f.data.lock().unwrap();
                    let res = Response::from_data(metrics.as_slice());
                    if let Err(e) = req.respond(res) {
                        eprintln!("metrics_server error: {}", e);
                    };
                }
            }
        }));

        MetricsServer { fields, thread }
    }

    /// Safely updates the data in a `MetricsServer` and returns the number of
    /// bytes written.
    ///
    /// This function is thread safe and protected by a mutex. It is safe
    /// to call concurrently from multiple threads.
    ///
    /// # Examples
    ///
    /// ```
    /// use metrics_server::MetricsServer;
    ///
    /// let server = MetricsServer::new("localhost:8001");
    /// let bytes = server.update(Vec::from([1, 2, 3, 4]));
    /// assert_eq!(4, bytes);
    /// ```
    pub fn update(&self, data: Vec<u8>) -> usize {
        let mut buf = self.fields.data.lock().unwrap();
        *buf = data;
        buf.as_slice().len()
    }
}

impl Drop for MetricsServer {
    // TODO: should I really be doing this inside drop? It _could_ panic,
    // so maybe a shutdown method would be better?
    fn drop(&mut self) {
        // Signal that we should stop handling requests and unblock the server.
        self.fields.stop.swap(true, Ordering::Relaxed);
        self.fields.server.unblock();

        // Because join takes ownership of the thread, we need call the take method
        // on the Option to move the value out of the Some variant and leave a None
        // variant in its place.
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
