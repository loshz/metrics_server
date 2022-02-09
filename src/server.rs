use std::thread;

use tiny_http::{Method, Response, Server};

const METHOD_NOT_FOUND: u16 = 404;
const METHOD_NOT_ALLOWED: u16 = 405;

pub struct MetricsServer {
    server: Server,
}

impl MetricsServer {
    pub fn new(addr: &str) -> MetricsServer {
        let server = tiny_http::Server::http(addr).unwrap();
        MetricsServer { server }
    }

    /// Starts a simple HTTP server on a new thread at the given address and expose the given metrics.
    /// This server is intended to only be queried synchronously as it blocks upon receiving
    /// each request.
    pub fn serve(self, data: Vec<u8>) {
        // Handle requests in a new thread so we can process in the background.
        thread::spawn(move || {
            loop {
                // Blocks until the next request is received.
                let req = match self.server.recv() {
                    Ok(req) => req,
                    Err(e) => {
                        eprintln!("error: {}", e);
                        continue;
                    }
                };

                // Only reponsd to GET requests.
                if req.method() != &Method::Get {
                    let res = Response::empty(METHOD_NOT_ALLOWED);
                    if let Err(e) = req.respond(res) {
                        eprintln!("{}", e);
                    };
                    continue;
                }

                // Only serve the /metrics path.
                if req.url() != "/metrics" {
                    let res = Response::empty(METHOD_NOT_FOUND);
                    if let Err(e) = req.respond(res) {
                        eprintln!("{}", e);
                    };
                    continue;
                }

                // Write the Prometheus metrics.
                let res = Response::from_data(&*data);
                if let Err(e) = req.respond(res) {
                    eprintln!("{}", e);
                };
            }
        });
    }
}
