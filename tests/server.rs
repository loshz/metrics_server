use metrics_server::{MetricsServer, ServerError};

#[test]
fn test_new_server_invalid_address() {
    let server = MetricsServer::new("invalid:99999999", None, None);
    assert!(server.is_err());
    assert!(matches!(server, Err(ServerError::Create(_))));
}

#[test]
fn test_new_http_server() {
    let server = MetricsServer::new("localhost:8001", None, None);
    assert!(server.is_ok());
}

#[test]
#[cfg(feature = "tls")]
fn test_new_server_invalid_certificate() {
    // Load TLS config.
    let cert = "-----BEGIN CERTIFICATE-----
invaid certificate
-----END CERTIFICATE-----"
        .as_bytes()
        .to_vec();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    let server = MetricsServer::new("localhost:8441", Some(cert), Some(key));
    assert!(server.is_err());
    assert!(matches!(server, Err(ServerError::Create(_))));
}

#[test]
#[cfg(feature = "tls")]
fn test_new_server_invalid_private_key() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = "-----BEGIN PRIVATE KEY-----
-----END PRIVATE KEY-----"
        .as_bytes()
        .to_vec();

    let server = MetricsServer::new("localhost:8442", Some(cert), Some(key));
    assert!(server.is_err());
    assert!(matches!(server, Err(ServerError::Create(_))));
}

#[test]
fn test_new_server_already_running() {
    let mut server = MetricsServer::new("localhost:8002", None, None).unwrap();

    // Attempt to start an already running server should be ok
    // as we will return the pre-existing thread.
    server.serve();
    server.serve();

    // Stop the server.
    server.stop().unwrap();
}

#[test]
fn test_new_https_server() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    let server = MetricsServer::new("localhost:8443", Some(cert), Some(key));
    assert!(server.is_ok());
}

#[test]
#[should_panic]
fn test_http_server_invalid_address() {
    _ = MetricsServer::http("invalid:99999999");
}

#[test]
fn test_http_server_serve() {
    let server = MetricsServer::http("localhost:8001");

    // Assert calls to non /metrics endpoint returns 404.
    let res = reqwest::blocking::get("http://localhost:8001/invalid").unwrap();
    assert_eq!(404, res.status());

    // Assert calls to URLs with correct prefix but additional characters returns 404.
    let res = reqwest::blocking::get("http://localhost:8001/metricsssss").unwrap();
    assert_eq!(404, res.status());

    // Assert calls to uppercase URLs returns 200.
    let res = reqwest::blocking::get("http://localhost:8001/METRICS").unwrap();
    assert_eq!(200, res.status());

    // Assert non GET requests to /metrics endpoint returns 405.
    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8001/metrics").send().unwrap();
    assert_eq!(405, res.status());

    // Assert calls to /metrics return correct response.
    for i in 0..3 {
        // Create mock data and update the metrics server.
        let v = vec![i];
        let b = server.update(v.clone());
        assert_eq!(1, b);

        // Make a HTTP request to the metrics server.
        let mut res = reqwest::blocking::get("http://localhost:8001/metrics").unwrap();
        assert_eq!(200, res.status());

        // Read the response body and check it is what we expected.
        let mut buf: Vec<u8> = vec![];
        res.copy_to(&mut buf).unwrap();
        assert_eq!(v, buf);
    }

    // Stop the server.
    server.stop().unwrap();
}

#[test]
fn test_http_server_serve_url() {
    let mut server = MetricsServer::new("localhost:8004", None, None).unwrap();
    server.serve_url("/test".to_string());

    // Assert calls to non /metrics endpoint returns 404.
    let res = reqwest::blocking::get("http://localhost:8004/metrics").unwrap();
    assert_eq!(404, res.status());

    // Assert calls to /test returns 200.
    let res = reqwest::blocking::get("http://localhost:8004/test").unwrap();
    assert_eq!(200, res.status());

    // Stop the server.
    server.stop().unwrap();
}

#[test]
#[should_panic]
#[cfg(feature = "tls")]
fn test_https_server_invalid_address() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    _ = MetricsServer::https("invalid:99999999", cert, key);
}

#[test]
#[should_panic]
#[cfg(feature = "tls")]
fn test_https_server_invalid_certificate() {
    // Load TLS config.
    let cert = Vec::new();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    _ = MetricsServer::https("localhost:8441", cert, key);
}

#[test]
#[should_panic]
#[cfg(feature = "tls")]
fn test_https_server_invalid_private_key() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = Vec::new();

    _ = MetricsServer::https("localhost:8442", cert, key);
}

#[test]
#[cfg(feature = "tls")]
fn test_https_server_serve() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    let server = MetricsServer::https("localhost:8443", cert, key);

    // Stop the server.
    server.stop().unwrap();
}
