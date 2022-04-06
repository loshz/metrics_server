use metrics_server::MetricsServer;

#[test]
#[should_panic]
fn test_http_server_invalid_address() {
    let _ = MetricsServer::new("invalid:99999999");
}

#[test]
fn test_http_server_serve() {
    let server = MetricsServer::new("localhost:8001");

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
}

#[test]
#[should_panic]
fn test_https_server_invalid_address() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    let _ = MetricsServer::https("invalid:99999999", cert, key);
}

#[test]
#[should_panic]
fn test_https_server_invalid_cert() {
    // Load TLS config.
    let cert = Vec::new();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    let _ = MetricsServer::https("localhost:8441", cert, key);
}

#[test]
#[should_panic]
fn test_https_server_invalid_key() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = Vec::new();

    let _ = MetricsServer::https("localhost:8442", cert, key);
}

#[test]
fn test_https_server_serve() {
    // Load TLS config.
    let cert = include_bytes!("./certs/certificate.pem").to_vec();
    let key = include_bytes!("./certs/private_key.pem").to_vec();

    let _ = MetricsServer::https("localhost:8443", cert, key);
}
