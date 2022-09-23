use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::registry::Registry;

use metrics_server::MetricsServer;

fn main() {
    // Register stop handler.
    let stop = Arc::new(AtomicBool::new(false));
    let s = stop.clone();
    ctrlc::set_handler(move || {
        s.store(true, Ordering::Relaxed);
    })
    .unwrap();

    // Expose the Prometheus metrics.
    let server = MetricsServer::http("localhost:8001");
    println!("Starting metrics server: http://localhost:8001/metrics");

    std::thread::scope(|s| {
        let handle = s.spawn(|| {
            // Create a metrics registry and counter that represents a single monotonically
            // increasing counter.
            let mut registry = Registry::default();
            let counter: Counter = Counter::default();
            registry.register("some_count", "Number of random counts", counter.clone());

            // Increment the counter every 5 seconds.
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }

                counter.inc();

                // Encode the current Registry in Prometheus format.
                let mut encoded = Vec::new();
                encode(&mut encoded, &registry).unwrap();

                // Update the Metrics Server with the current encoded data.
                server.update(encoded);

                thread::sleep(time::Duration::from_secs(5));
            }
        });

        handle.join().unwrap();
    });

    // Stop server.
    server.stop().unwrap();
}
