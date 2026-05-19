use axum::{
    Router,
    extract::{Path, Query},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use identicon_core::{RenderError, RenderOptions, Theme};
use std::env;

#[derive(Debug, Default, serde::Deserialize)]
struct IdenticonQuery {
    size: Option<u32>,
    theme: Option<String>,
    background: Option<bool>,
    animated: Option<bool>,
}

async fn identicon_handler(
    Path(input): Path<String>,
    Query(query): Query<IdenticonQuery>,
) -> Response {
    let theme = match query.theme.as_deref() {
        Some(name) => match Theme::from_name(name) {
            Some(t) => Some(t),
            None => {
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

    match identicon_core::render_identicon_with_options(&input, &opts) {
        Ok(svg) => {
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
            (StatusCode::BAD_REQUEST, "input must not be empty").into_response()
        }
        Err(RenderError::InputTooLong) => {
            (StatusCode::BAD_REQUEST, "input too long").into_response()
        }
        Err(RenderError::InvalidCharset) => {
            (StatusCode::BAD_REQUEST, "invalid characters in input").into_response()
        }
    }
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

#[tokio::main]
async fn main() {
    let app = Router::new().route("/{input}", get(identicon_handler));

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    eprintln!("identicon-api listening on http://{addr}");
    axum::serve(listener, app).await.expect("serve");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn returns_svg_with_cache_headers() {
        let app = Router::new().route("/{input}", get(identicon_handler));
        let response = app
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
        let app = Router::new().route("/{input}", get(identicon_handler));

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
