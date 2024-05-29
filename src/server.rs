use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use http::uri::PathAndQuery;
use log::{debug, error};
use time::{format_description, OffsetDateTime};
use tiny_http::{ConfigListenAddr, Method, Response, Server};

use crate::error::ServerError;

/// The default metrics URL path of the server.
pub const DEFAULT_METRICS_PATH: &str = "/metrics";

/// A thread-safe datastore for serving metrics via a HTTP/S server.
pub struct MetricsServer {
    shared: Arc<SharedData>,
    thread: Option<thread::JoinHandle<()>>,
}

struct SharedData {
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
        let shared = Arc::new(SharedData {
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

    /// Thread safe method for updating the data in a `MetricsServer`, returning the number of bytes written.
    pub fn update(&self, data: Vec<u8>) -> usize {
        let mut buf = self.shared.data.lock().unwrap();
        *buf = data;
        buf.len()
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
    pub fn serve_uri(&mut self, path: String) {
        // Check if we already have a thread running.
        if let Some(thread) = &self.thread {
            if !thread.is_finished() {
                debug!("metrics server already running, continuing");
                return;
            }
        }

        // Ensure path is valid.
        let path = parse_path(&path);

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
                        debug!("metrics server stopping");
                        return;
                    }

                    // Only serve the specified URI path.
                    if req.url() != path {
                        let res = Response::empty(404);
                        respond(req, res);
                        continue;
                    }

                    // Only respond to GET requests.
                    if req.method() != &Method::Get {
                        let res = Response::empty(405);
                        respond(req, res);
                        continue;
                    }

                    // Write the metrics to the response buffer.
                    let metrics = s.data.lock().unwrap();
                    let res = Response::from_data(metrics.as_slice());
                    respond(req, res);
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

// Validate the provided URL path, or return the default path on error.
fn parse_path(uri: &str) -> String {
    match PathAndQuery::from_str(uri) {
        Ok(pq) => {
            let mut path = pq.path().to_lowercase();
            if !path.starts_with('/') {
                path.insert(0, '/');
            }
            path
        }
        Err(_) => {
            error!("invalid uri, defaulting to {DEFAULT_METRICS_PATH}");
            DEFAULT_METRICS_PATH.to_string()
        }
    }
}

// Responds to a given request and logs in an Apache-like format.
fn respond<D>(req: tiny_http::Request, res: tiny_http::Response<D>)
where
    D: std::io::Read,
{
    let datetime = OffsetDateTime::now_utc()
        .format(&format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "-".to_string());

    debug!(
        "{} [{}] \"{} {} HTTP/{}\" {}",
        req.remote_addr().map_or("-".to_string(), |v| v.to_string()),
        datetime,
        req.method(),
        req.url(),
        req.http_version(),
        res.status_code().0,
    );

    if let Err(e) = req.respond(res) {
        error!("error sending metrics response: {e}");
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let expected_default = DEFAULT_METRICS_PATH.to_string();
        let expected_valid = "/debug/metrics".to_string();

        // Invalid.
        assert_eq!(parse_path("Hello, World!"), expected_default);
        // Whitespace.
        assert_eq!(parse_path(" metr ics  "), expected_default);
        // Non-ASCII.
        assert_eq!(parse_path("mëtrîcs"), expected_default);
        // Valid.
        assert_eq!(parse_path("/debug/metrics"), expected_valid);
        assert_eq!(parse_path("debug/metrics"), expected_valid);
        assert_eq!(parse_path("DEBUG/METRICS"), expected_valid);
    }
}
