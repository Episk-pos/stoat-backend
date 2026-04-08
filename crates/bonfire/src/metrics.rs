use prometheus::{
    Counter, CounterVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
};
use std::sync::OnceLock;

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get_or_init(|| {
        let r = Registry::new();
        register_metrics(&r);
        r
    })
}

fn register_metrics(registry: &Registry) {
    registry
        .register(Box::new(WEBSOCKET_CONNECTIONS_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(WEBSOCKET_CONNECTIONS_ACTIVE.clone()))
        .unwrap();
    registry
        .register(Box::new(WEBSOCKET_MESSAGES_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(WEBSOCKET_MESSAGE_SIZE_BYTES.clone()))
        .unwrap();
    registry
        .register(Box::new(WEBSOCKET_CONNECTION_DURATION_SECONDS.clone()))
        .unwrap();
    registry
        .register(Box::new(WEBSOCKET_AUTHENTICATION_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(REDIS_EVENTS_TOTAL.clone()))
        .unwrap();
}

lazy_static::lazy_static! {
    /// Total number of WebSocket connections established
    pub static ref WEBSOCKET_CONNECTIONS_TOTAL: Counter = Counter::new(
        "bonfire_websocket_connections_total",
        "Total number of WebSocket connections established"
    ).unwrap();

    /// Current number of active WebSocket connections
    pub static ref WEBSOCKET_CONNECTIONS_ACTIVE: Counter = Counter::new(
        "bonfire_websocket_connections_active",
        "Current number of active WebSocket connections"
    ).unwrap();

    /// Total number of WebSocket messages by direction
    pub static ref WEBSOCKET_MESSAGES_TOTAL: CounterVec = CounterVec::new(
        Opts::new("bonfire_websocket_messages_total", "Total number of WebSocket messages"),
        &["direction", "message_type"]
    ).unwrap();

    /// WebSocket message size distribution in bytes
    pub static ref WEBSOCKET_MESSAGE_SIZE_BYTES: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "bonfire_websocket_message_size_bytes",
            "WebSocket message size distribution in bytes"
        ).buckets(vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0, 100000.0]),
        &["direction"]
    ).unwrap();

    /// WebSocket connection duration in seconds
    pub static ref WEBSOCKET_CONNECTION_DURATION_SECONDS: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "bonfire_websocket_connection_duration_seconds",
            "WebSocket connection duration in seconds"
        ).buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0, 7200.0])
    ).unwrap();

    /// Total number of authentication attempts by result
    pub static ref WEBSOCKET_AUTHENTICATION_TOTAL: CounterVec = CounterVec::new(
        Opts::new("bonfire_websocket_authentication_total", "Total number of authentication attempts"),
        &["result"]  // "success" or "failure"
    ).unwrap();

    /// Total number of Redis events processed
    pub static ref REDIS_EVENTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("bonfire_redis_events_total", "Total number of Redis events processed"),
        &["event_type"]
    ).unwrap();
}
