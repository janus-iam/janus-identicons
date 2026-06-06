mod metrics;

use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use identicon_core::{RenderError, RenderOptions, Theme};
use metrics::{Metrics, route_label};
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::Level;

#[derive(Clone)]
struct AppState {
    metrics: Arc<Metrics>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct IdenticonQuery {
    size: Option<u32>,
    theme: Option<String>,
    background: Option<bool>,
    animated: Option<bool>,
}

async fn health_handler() -> impl IntoResponse {
    axum::Json(serde_json::json!({ "status": "ok" }))
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
        )],
        state.metrics.encode(),
    )
}

async fn identicon_handler(
    State(state): State<AppState>,
    Path(input): Path<String>,
    Query(query): Query<IdenticonQuery>,
) -> Response {
    let theme = match query.theme.as_deref() {
        Some(name) => match Theme::from_name(name) {
            Some(t) => Some(t),
            None => {
                state.metrics.record_render(400, Duration::ZERO, 0);
                return (
                    StatusCode::BAD_REQUEST,
                    "unknown theme; valid: aurora, sunset, synthwave, nord, monochrome, oceanic, neon, pastel",
                )
                    .into_response();
            }
        },
        None => None,
    };

    let opts = RenderOptions {
        size: query.size.unwrap_or(256),
        theme,
        background: query.background.unwrap_or(true),
        animated: query.animated.unwrap_or(false),
    };

    let started = Instant::now();
    match identicon_core::render_identicon_with_options(&input, &opts) {
        Ok(svg) => {
            let elapsed = started.elapsed();
            state.metrics.record_render(200, elapsed, svg.len());
            let payload = etag_payload(&input, &opts);
            let etag = format!("\"{}\"", blake3::hash(payload.as_bytes()).to_hex());
            (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("image/svg+xml; charset=utf-8"),
                    ),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, immutable, max-age=31536000"),
                    ),
                    (
                        header::ETAG,
                        HeaderValue::from_str(&etag).unwrap_or(HeaderValue::from_static("\"0\"")),
                    ),
                ],
                svg,
            )
                .into_response()
        }
        Err(RenderError::EmptyInput) => {
            state.metrics.record_render(400, started.elapsed(), 0);
            (StatusCode::BAD_REQUEST, "input must not be empty").into_response()
        }
        Err(RenderError::InputTooLong) => {
            state.metrics.record_render(400, started.elapsed(), 0);
            (StatusCode::BAD_REQUEST, "input too long").into_response()
        }
        Err(RenderError::InvalidCharset) => {
            state.metrics.record_render(400, started.elapsed(), 0);
            (StatusCode::BAD_REQUEST, "invalid characters in input").into_response()
        }
    }
}

async fn track_requests(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_string();
    let route = route_label(request.uri().path()).to_string();
    let response = next.run(request).await;
    state
        .metrics
        .record_request(&method, &route, response.status().as_u16());
    response
}

fn etag_payload(input: &str, opts: &RenderOptions) -> String {
    let theme = opts
        .theme
        .map(|t| t.index().to_string())
        .unwrap_or_else(|| "auto".to_string());
    format!(
        "{}|{}|{}|{}|{}",
        input, opts.size, theme, opts.background, opts.animated
    )
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "identicon_api=info,tower_http=info".into()),
        )
        .init();
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/{input}", get(identicon_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            track_requests,
        ))
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                |request: &Request<Body>| {
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        route = route_label(request.uri().path()),
                    )
                },
            ),
        )
        .with_state(state)
}

#[tokio::main]
async fn main() {
    init_tracing();

    let state = AppState {
        metrics: Arc::new(Metrics::new()),
    };
    let app = app(state);

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    tracing::info!(%addr, "identicon-api listening");
    axum::serve(listener, app).await.expect("serve");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_app() -> Router {
        app(AppState {
            metrics: Arc::new(Metrics::new()),
        })
    }

    #[tokio::test]
    async fn health_returns_ok_json() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn metrics_returns_prometheus_text() {
        let app = test_app();
        app.clone()
            .oneshot(Request::get("/alice").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("identicon_requests_total"));
        assert!(text.contains("identicon_render_duration_seconds"));
        assert!(text.contains("identicon_svg_size_bytes"));
    }

    #[tokio::test]
    async fn returns_svg_with_cache_headers() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/alice")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "public, immutable, max-age=31536000"
        );
        assert!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("image/svg+xml")
        );
    }

    #[tokio::test]
    async fn query_size_changes_output() {
        let app = test_app();

        let default = app
            .clone()
            .oneshot(Request::get("/alice").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let resized = app
            .oneshot(Request::get("/alice?size=128").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let default_body = axum::body::to_bytes(default.into_body(), usize::MAX)
            .await
            .unwrap();
        let resized_body = axum::body::to_bytes(resized.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_ne!(default_body, resized_body);
    }
}
