use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Method, Response, Server};

#[derive(Clone)]
pub struct MetricsServer(Arc<Mutex<Vec<u8>>>);

impl Write for MetricsServer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // TODO: this seems like a hack.
        // Ideally, I want to lock before the loop and unlock afterwards.
        for b in buf.iter() {
            (*self.0.lock().unwrap()).push(*b);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        (*self.0.lock().unwrap()).clear();
        Ok(())
    }
}

impl Default for MetricsServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsServer {
    pub fn new() -> Self {
        MetricsServer(Arc::new(Mutex::new(Vec::new())))
    }

    /// Starts a simple HTTP server on a new thread at the given address and expose the stored metrics.
    /// This server is intended to only be queried synchronously as it blocks upon receiving
    /// each request.
    pub fn serve(self, addr: &str) {
        let server = Server::http(addr).unwrap();

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

                    // Only reponsd to GET requests(?).
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

                    // Write the Prometheus metrics.
                    let res = Response::from_data("");
                    if let Err(e) = req.respond(res) {
                        eprintln!("{}", e);
                    };
                }
            }
        });
    }
}
