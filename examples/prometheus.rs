use std::{thread, time};

use metrics_server::MetricsServer;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::registry::Registry;

fn main() {
    // Create a metrics registry.
    let mut registry = Registry::default();

    // Create a metric that represents a single monotonically increasing counter.
    let counter: Counter = Counter::default();

    // Register a metric with the Registry.
    registry.register("some_count", "Number of random counts", counter.clone());

    // Expose the Prometheus metrics.
    let server = MetricsServer::new();
    server.serve("localhost:8001");

    // Increment the counter every 5 seconds.
    loop {
        counter.inc();

        // Encode the current Registry in Prometheus format.
        let mut encoded = Vec::new();
        encode(&mut encoded, &registry).unwrap();

        // Update the Metrics Server with the current encoded data.
        server.update(encoded);

        let ten_secs = time::Duration::from_secs(5);
        thread::sleep(ten_secs);
    }
}
