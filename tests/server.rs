use metrics_server::MetricsServer;

#[test]
#[should_panic]
fn test_server_serve() {
    let server = MetricsServer::new();
    server.serve("invalid:99999999");
}
