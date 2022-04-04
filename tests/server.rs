use metrics_server::MetricsServer;

#[test]
#[should_panic]
fn test_server_invalid_address() {
    let _ = MetricsServer::new("invalid:99999999");
}

#[test]
fn test_server_serve() {
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
