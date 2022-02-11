use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Method, Response, Server};

#[derive(Clone)]
pub struct MetricsServer(Arc<Mutex<Vec<u8>>>);

impl Default for MetricsServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsServer {
    pub fn new() -> Self {
        MetricsServer(Arc::new(Mutex::new(Vec::new())))
    }

    pub fn update(&self, data: Vec<u8>) {
        let mut buf = self.0.lock().unwrap();
        *buf = data;
    }

    /// Starts a simple HTTP server on a new thread at the given address and expose the stored metrics.
    /// This server is intended to only be queried synchronously as it blocks upon receiving
    /// each request.
    pub fn serve(&self, addr: &str) {
        let server = Server::http(addr).unwrap();
        let buf = Arc::clone(&self.0);

        // Handle requests in a new thread so we can process in the background.
        thread::spawn({
            move || {
                loop {
                    // Blocks until the next request is received.
                    let req = match server.recv() {
                        Ok(req) => req,
                        Err(e) => {
                            eprintln!("error: {}", e);
                            continue;
                        }
                    };

                    // Only respond to GET requests(?).
                    if req.method() != &Method::Get {
                        let res = Response::empty(405);
                        if let Err(e) = req.respond(res) {
                            eprintln!("{}", e);
                        };
                        continue;
                    }

                    // TODO: this is naive. Fix it(?)
                    // Only serve the /metrics path.
                    if req.url() != "/metrics" {
                        let res = Response::empty(404);
                        if let Err(e) = req.respond(res) {
                            eprintln!("{}", e);
                        };
                        continue;
                    }

                    // Write the metrics to the response buffer.
                    let metrics = buf.lock().unwrap();
                    let res = Response::from_data(metrics.as_slice());
                    if let Err(e) = req.respond(res) {
                        eprintln!("{}", e);
                    };
                }
            }
        });
    }
}
