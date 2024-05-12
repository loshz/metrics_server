use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use log::info;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::registry::Registry;

use metrics_server::MetricsServer;

fn main() {
    env_logger::init();

    // Register stop handler.
    let (send, recv) = mpsc::channel();
    ctrlc::set_handler(move || {
        info!("Stopping metrics server");
        send.send(()).unwrap();
    })
    .unwrap();

    // Expose the Prometheus metrics.
    let server = MetricsServer::http("localhost:8001");
    info!("Starting metrics server: http://localhost:8001/metrics");

    thread::scope(|s| {
        let handle = s.spawn(move || {
            // Create a metrics registry and counter that represents a single monotonically
            // increasing counter.
            let mut registry = Registry::default();
            let counter: Counter = Counter::default();
            registry.register("some_count", "Number of random counts", counter.clone());

            // Increment the counter periodically.
            loop {
                counter.inc();

                // Encode the current Registry in Prometheus format
                let mut encoded = String::new();
                encode(&mut encoded, &registry).unwrap();

                // Update the Metrics Server with the current encoded data.
                server.update(encoded.into());

                // Sleep for 5 seconds or exit.
                if recv.recv_timeout(Duration::from_secs(5)).is_ok() {
                    // Stop server.
                    server.stop().unwrap();
                    break;
                }
            }
        });

        handle.join().unwrap();
    });
}
