use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
use std::time::Duration;

pub struct Metrics {
    pub registry: Registry,
    requests_total: IntCounterVec,
    render_duration_seconds: HistogramVec,
    svg_size_bytes: HistogramVec,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let requests_total = IntCounterVec::new(
            Opts::new("identicon_requests_total", "Total HTTP requests"),
            &["method", "route", "status"],
        )
        .expect("identicon_requests_total metric");

        let render_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "identicon_render_duration_seconds",
                "Time spent rendering identicons",
            )
            .buckets(vec![
                0.000_01, 0.000_05, 0.000_1, 0.000_5, 0.001, 0.005, 0.01, 0.05, 0.1,
            ]),
            &["status"],
        )
        .expect("identicon_render_duration_seconds metric");

        let svg_size_bytes = HistogramVec::new(
            HistogramOpts::new(
                "identicon_svg_size_bytes",
                "Rendered SVG payload size in bytes",
            )
            .buckets(prometheus::exponential_buckets(512.0, 2.0, 8).expect("buckets")),
            &["status"],
        )
        .expect("identicon_svg_size_bytes metric");

        registry
            .register(Box::new(requests_total.clone()))
            .expect("register identicon_requests_total");
        registry
            .register(Box::new(render_duration_seconds.clone()))
            .expect("register identicon_render_duration_seconds");
        registry
            .register(Box::new(svg_size_bytes.clone()))
            .expect("register identicon_svg_size_bytes");

        Self {
            registry,
            requests_total,
            render_duration_seconds,
            svg_size_bytes,
        }
    }

    pub fn record_request(&self, method: &str, route: &str, status: u16) {
        self.requests_total
            .with_label_values(&[method, route, &status.to_string()])
            .inc();
    }

    pub fn record_render(&self, status: u16, duration: Duration, svg_len: usize) {
        let status = status.to_string();
        self.render_duration_seconds
            .with_label_values(&[&status])
            .observe(duration.as_secs_f64());
        if svg_len > 0 {
            self.svg_size_bytes
                .with_label_values(&[&status])
                .observe(svg_len as f64);
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .expect("encode prometheus metrics");
        buffer
    }
}

pub fn route_label(path: &str) -> &'static str {
    match path {
        "/health" => "/health",
        "/metrics" => "/metrics",
        _ => "/{input}",
    }
}
