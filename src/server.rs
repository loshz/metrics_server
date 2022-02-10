use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Method, Response, Server};

const METHOD_NOT_FOUND: u16 = 404;
const METHOD_NOT_ALLOWED: u16 = 405;

#[derive(Clone)]
pub struct MetricsServer<T>(Arc<Mutex<T>>);

impl<W: Write> Write for MetricsServer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (*self.0.lock().unwrap()).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (*self.0.lock().unwrap()).flush()
    }
}

impl<R: Read> Read for MetricsServer<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (*self.0.lock().unwrap()).read(buf)
    }
}

impl<RW: Read + Write> MetricsServer<RW> {
    pub fn new(rw: RW) -> Self {
        MetricsServer(Arc::new(Mutex::new(rw)))
    }

    /// Starts a simple HTTP server on a new thread at the given address and expose the given metrics.
    /// This server is intended to only be queried synchronously as it blocks upon receiving
    /// each request.
    pub fn serve(self, addr: &str) {
        let server = Server::http(addr).unwrap();

        // Handle requests in a new thread so we can process in the background.
        thread::spawn({
            //let buff = Arc::clone(&self.buffer);

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

                    // Only reponsd to GET requests.
                    if req.method() != &Method::Get {
                        let res = Response::empty(METHOD_NOT_ALLOWED);
                        if let Err(e) = req.respond(res) {
                            eprintln!("{}", e);
                        };
                        continue;
                    }

                    // TODO: this is naive. Fix it!
                    // Only serve the /metrics path.
                    if req.url() != "/metrics" {
                        let res = Response::empty(METHOD_NOT_FOUND);
                        if let Err(e) = req.respond(res) {
                            eprintln!("{}", e);
                        };
                        continue;
                    }

                    // TODO: is this a data race?
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
