use axum::{
    extract::{MatchedPath, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use prometheus::{
    opts, register_histogram_vec, register_int_counter_vec, Encoder, HistogramOpts, HistogramVec,
    IntCounterVec, TextEncoder,
};
use std::time::Instant;

static HTTP_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        opts!("http_requests_total", "Total HTTP requests"),
        &["method", "path", "status"]
    )
    .expect("register http_requests_total")
});

static HTTP_REQUEST_DURATION_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ),
        &["method", "path", "status"]
    )
    .expect("register http_request_duration_seconds")
});

fn ensure_metrics_registered() {
    Lazy::force(&HTTP_REQUESTS_TOTAL);
    Lazy::force(&HTTP_REQUEST_DURATION_SECONDS);
}

pub async fn metrics() -> Result<(StatusCode, String), (StatusCode, String)> {
    ensure_metrics_registered();

    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let body = String::from_utf8(buffer)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::OK, body))
}

pub async fn track_metrics(req: Request, next: Next) -> Response {
    ensure_metrics_registered();

    let start = Instant::now();
    let method = req.method().as_str().to_owned();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());

    let response = next.run(req).await;
    let status = response.status().as_u16().to_string();
    let elapsed = start.elapsed().as_secs_f64();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();
    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[&method, &path, &status])
        .observe(elapsed);

    response
}
