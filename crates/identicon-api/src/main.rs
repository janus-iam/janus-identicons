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
use identicon_core::{Engine, RenderError, RenderOptions, Theme};
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
    engine: Option<String>,
}

const GALLERY_PEOPLE: [&str; 5] = ["alice", "bob", "carol", "diego", "emma"];

fn comparison_page() -> String {
    let mut html = String::with_capacity(48 * 1024);
    html.push_str(
        r#"<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Comparaison des identicons</title><style>
body{margin:0;padding:28px 32px 64px;background:#111318;color:#e8eaed;font-family:Inter,system-ui,sans-serif}
h1{margin:0 0 6px;font-size:28px} .lead{margin:0 0 28px;color:#9aa0a6;max-width:46rem;line-height:1.45}
section{margin-bottom:36px} h2{margin:0 0 8px;font-size:20px;text-transform:capitalize;position:sticky;top:0;background:#111318;padding:8px 0}
table{border-collapse:separate;border-spacing:12px 14px;margin-left:-12px}
th{font-weight:600;text-align:center;color:#9aa0a6;font-size:13px} th.theme{text-align:left;color:#e8eaed;font-size:15px;text-transform:capitalize;width:8rem}
img{width:104px;height:104px;border-radius:14px;display:block;background:#1c1f27}
</style></head><body><h1>Comparaison des identicons</h1><p class="lead">Cinq personnes pour chaque thème et chaque moteur. Chaque image est servie par l'API, par exemple <code>/alice?engine=ribbon&amp;theme=nord&amp;size=104</code>. Sans <code>engine</code>, l'API utilise <code>ribbon</code>.</p>"#,
    );
    for engine in Engine::ALL {
        let name = engine.name();
        html.push_str(&format!(
            r#"<section id="{name}"><h2>{name}</h2><table><thead><tr><th></th>"#
        ));
        for person in GALLERY_PEOPLE {
            html.push_str(&format!("<th>{person}</th>"));
        }
        html.push_str("</tr></thead><tbody>");
        for palette in identicon_core::PALETTES {
            let theme = palette.name;
            html.push_str(&format!(r#"<tr><th class="theme">{theme}</th>"#));
            for person in GALLERY_PEOPLE {
                html.push_str(&format!(
                    r#"<td><img alt="{person}, moteur {name}, thème {theme}" width="104" height="104" loading="lazy" src="/{person}?engine={name}&theme={theme}&size=104"></td>"#
                ));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table></section>");
    }
    html.push_str("</body></html>");
    html
}

async fn gallery_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            ),
            (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
        ],
        comparison_page(),
    )
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
                    unknown_theme_message(),
                )
                    .into_response();
            }
        },
        None => None,
    };

    let engine = match query.engine.as_deref() {
        Some(name) => match Engine::from_name(name) {
            Some(engine) => engine,
            None => {
                state.metrics.record_render(400, Duration::ZERO, 0);
                return (
                    StatusCode::BAD_REQUEST,
                    "unknown engine; valid: blob, crest, voronoi, kaleido, ribbon, constellation, monogram",
                )
                    .into_response();
            }
        },
        None => Engine::Ribbon,
    };

    let opts = RenderOptions {
        size: query.size.unwrap_or(256),
        theme,
        background: query.background.unwrap_or(true),
        animated: query.animated.unwrap_or(false),
        engine,
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

fn unknown_theme_message() -> String {
    let names = identicon_core::PALETTES
        .iter()
        .map(|palette| palette.name)
        .collect::<Vec<_>>()
        .join(", ");
    format!("unknown theme; valid: {names}")
}

fn etag_payload(input: &str, opts: &RenderOptions) -> String {
    let theme = opts
        .theme
        .map(|t| t.index().to_string())
        .unwrap_or_else(|| "auto".to_string());
    format!(
        "{}|{}|{}|{}|{}|{}",
        input,
        opts.size,
        theme,
        opts.background,
        opts.animated,
        opts.engine.name()
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
        .route("/", get(gallery_handler))
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

    #[tokio::test]
    async fn engine_query_changes_output_and_rejects_unknown() {
        let app = test_app();

        let omitted = app
            .clone()
            .oneshot(Request::get("/alice").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let ribbon = app
            .clone()
            .oneshot(
                Request::get("/alice?engine=ribbon")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let blob = app
            .clone()
            .oneshot(
                Request::get("/alice?engine=blob")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let crest = app
            .clone()
            .oneshot(
                Request::get("/alice?engine=crest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let unknown = app
            .oneshot(
                Request::get("/alice?engine=nope")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(omitted.status(), StatusCode::OK);
        assert_eq!(ribbon.status(), StatusCode::OK);
        assert_eq!(blob.status(), StatusCode::OK);
        assert_eq!(crest.status(), StatusCode::OK);
        assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            omitted.headers().get(header::ETAG).cloned(),
            ribbon.headers().get(header::ETAG).cloned()
        );
        assert_ne!(
            blob.headers().get(header::ETAG).cloned(),
            crest.headers().get(header::ETAG).cloned()
        );

        let omitted_body = axum::body::to_bytes(omitted.into_body(), usize::MAX)
            .await
            .unwrap();
        let ribbon_body = axum::body::to_bytes(ribbon.into_body(), usize::MAX)
            .await
            .unwrap();
        let blob_body = axum::body::to_bytes(blob.into_body(), usize::MAX)
            .await
            .unwrap();
        let crest_body = axum::body::to_bytes(crest.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(omitted_body, ribbon_body);
        assert_ne!(omitted_body, blob_body);
        assert_ne!(blob_body, crest_body);
    }

    #[tokio::test]
    async fn index_compares_every_theme_and_engine() {
        let app = test_app();
        let response = app
            .clone()
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();

        let mut sources = Vec::new();
        let mut rest = html.as_str();
        while let Some(start) = rest.find("src=\"") {
            rest = &rest[start + 5..];
            let end = rest.find('"').unwrap();
            sources.push(rest[..end].to_string());
            rest = &rest[end..];
        }
        assert_eq!(
            sources.len(),
            Engine::ALL.len() * identicon_core::PALETTES.len() * GALLERY_PEOPLE.len()
        );

        for engine in Engine::ALL {
            assert!(html.contains(&format!("id=\"{}\"", engine.name())));
        }
        for palette in identicon_core::PALETTES {
            assert!(html.contains(palette.name));
        }
        for person in GALLERY_PEOPLE {
            assert!(html.contains(&format!("<th>{person}</th>")));
        }

        for src in sources {
            let image = app
                .clone()
                .oneshot(Request::get(&src).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(image.status(), StatusCode::OK, "{src}");
            assert!(
                image
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("image/svg+xml"),
                "{src}"
            );
        }
    }
}
