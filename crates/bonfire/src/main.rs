use std::env;

use async_std::net::TcpListener;
use revolt_presence::clear_region;

#[macro_use]
extern crate log;

pub mod config;
pub mod events;
pub mod metrics;

mod database;
mod websocket;

#[async_std::main]
async fn main() {
    // Configure requirements for Bonfire.
    revolt_config::configure!(events);
    database::connect().await;

    // Clean up the current region information.
    let no_clear_region = env::var("NO_CLEAR_PRESENCE").unwrap_or_else(|_| "0".into()) == "1";
    if !no_clear_region {
        clear_region(None).await;
    }

    // Initialize metrics registry
    let _ = metrics::registry();

    // Start metrics HTTP server in background
    let metrics_bind = env::var("METRICS_HOST").unwrap_or_else(|_| "0.0.0.0:9090".into());
    info!("Starting metrics server on {metrics_bind}");
    async_std::task::spawn(metrics_server(metrics_bind));

    // Setup a TCP listener to accept WebSocket connections on.
    // By default, we bind to port 14703 on all interfaces.
    let bind = env::var("HOST").unwrap_or_else(|_| "0.0.0.0:14703".into());
    info!("Listening on host {bind}");
    let try_socket = TcpListener::bind(bind).await;
    let listener = try_socket.expect("Failed to bind");

    // Start accepting new connections and spawn a client for each connection.
    while let Ok((stream, addr)) = listener.accept().await {
        async_std::task::spawn(async move {
            info!("User connected from {addr:?}");
            websocket::client(database::get_db(), stream, addr).await;
            info!("User disconnected from {addr:?}");
        });
    }
}

async fn metrics_server(bind: String) {
    use async_std::io::WriteExt;
    use async_std::net::TcpListener;
    use async_std::prelude::*;
    use prometheus::{Encoder, TextEncoder};

    let listener = match TcpListener::bind(&bind).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind metrics server to {bind}: {e}");
            return;
        }
    };

    info!("Metrics server listening on {bind}");

    while let Ok((mut stream, _)) = listener.accept().await {
        async_std::task::spawn(async move {
            let mut buffer = vec![0u8; 1024];
            if stream.read(&mut buffer).await.is_err() {
                return;
            }

            // Simple HTTP response with metrics
            let encoder = TextEncoder::new();
            let metric_families = metrics::registry().gather();
            let mut metrics_buffer = vec![];
            if encoder.encode(&metric_families, &mut metrics_buffer).is_err() {
                return;
            }

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n",
                metrics_buffer.len()
            );

            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.write_all(&metrics_buffer).await;
            let _ = stream.flush().await;
        });
    }
}
