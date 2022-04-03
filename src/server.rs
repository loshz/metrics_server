use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Method, Response, Server};

/// A thread-safe datastore for serving metrics via a HTTP server.
pub struct MetricsServer {
    data: Arc<Mutex<Vec<u8>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Default for MetricsServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsServer {
    /// Creates a new empty `MetricsServer`.
    ///
    /// This will create a mutex protected empty Vector. It will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use metrics_server::MetricsServer;
    ///
    /// let server = MetricsServer::new();
    /// ```
    pub fn new() -> Self {
        MetricsServer {
            data: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::new(AtomicBool::new(false)),
            thread: None,
        }
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
    /// let mut server = MetricsServer::new();
    /// let bytes = server.update(Vec::from([1, 2, 3, 4]));
    /// assert_eq!(4, bytes);
    /// ```
    pub fn update(&self, data: Vec<u8>) -> usize {
        let mut buf = self.data.lock().unwrap();
        *buf = data;
        buf.as_slice().len()
    }

    /// Starts a simple HTTP server on a new thread at the given address and expose the stored metrics.
    /// This server is intended to only be queried synchronously as it blocks upon receiving
    /// each request.
    ///
    /// # Examples
    ///
    /// ```
    /// use metrics_server::MetricsServer;
    ///
    /// let mut server = MetricsServer::new();
    /// server.serve("localhost:8001");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if given an invalid address.
    pub fn serve(&mut self, addr: &str) {
        // Create a new HTTP server and bind to the given address.
        let server = Server::http(addr).unwrap();

        // Invoking clone on Arc produces a new Arc instance, which points to the
        // same allocation on the heap as the source Arc, while increasing a reference count.
        let buf = Arc::clone(&self.data);

        let stop = self.stop.clone();

        // Handle requests in a new thread so we can process in the background.
        let thread = thread::spawn({
            move || {
                for req in server.incoming_requests() {
                    // Check to see if we should stop handling requests.
                    if stop.load(Ordering::Relaxed) {
                        server.unblock();
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
                    let metrics = buf.lock().unwrap();
                    let res = Response::from_data(metrics.as_slice());
                    if let Err(e) = req.respond(res) {
                        eprintln!("metrics_server error: {}", e);
                    };
                }
            }
        });

        self.thread = Some(thread);
    }
}

impl Drop for MetricsServer {
    fn drop(&mut self) {
        // Signal that we should stop handling requests.
        self.stop.swap(true, Ordering::Relaxed);

        // Because join takes ownership of the thread, we need call the take method
        // on the Option to move the value out of the Some variant and leave a None
        // variant in its place.
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
