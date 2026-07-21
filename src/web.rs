use crate::console::Screen;
use crate::spotify;
use crate::token::Client;
use axum::extract::{Form, Query, State};
use axum::http::{header, HeaderMap, Uri};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, serve};
use oauth2::PkceCodeVerifier;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::watch::{Receiver, Sender};

#[derive(Deserialize)]
struct Input {
    uri: String,
}

#[derive(Deserialize)]
struct CallbackParameters {
    code: String,
}

#[derive(Clone)]
struct PlayerState {
    sender: Sender<Option<String>>,
    _receiver: Receiver<Option<String>>,
    oauth: Client,
    screen: Screen,
    code_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
    client: spotify::Client,
}

impl PlayerState {
    fn new(
        sender: Sender<Option<String>>,
        _receiver: Receiver<Option<String>>,
        oauth: Client,
        screen: Screen,
        client: spotify::Client,
    ) -> Self {
        Self {
            sender,
            _receiver,
            oauth,
            screen,
            client,
            code_verifier: Arc::new(Mutex::new(None)),
        }
    }
}

pub async fn run(
    sender: Sender<Option<String>>,
    receiver: Receiver<Option<String>>,
    oauth: Client,
    address: String,
    screen: Screen,
    client: spotify::Client,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address.as_str()).await?;
    let app = axum::Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/logs", get(logs))
        .route("/play", post(play).put(play))
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/devices", get(devices))
        .route("/authorization", get(authorization))
        .fallback(not_found)
        .with_state(PlayerState::new(sender, receiver, oauth, screen, client));

    tracing::debug!(%address, "listening to HTTP requests");

    serve(listener, app).await?;

    Ok(())
}

async fn logs(State(state): State<PlayerState>) -> Html<String> {
    Html(state.screen.read())
}

async fn play(State(state): State<PlayerState>, Form(input): Form<Input>) -> impl IntoResponse {
    let value = Some(input.uri).filter(|v| !v.is_empty());

    if let Err(e) = state.sender.send(value) {
        tracing::error!(%e, "Failed to set desired state as playing");
    }

    Redirect::to("/")
}

async fn devices(State(mut state): State<PlayerState>) -> Response {
    match state.client.get_available_devices().await {
        Ok(devices) => Json(devices).into_response(),
        Err(e) => Json(e.to_string()).into_response(),
    }
}

async fn authorization(State(mut state): State<PlayerState>) -> Json<String> {
    match state.oauth.authorization().await {
        Ok(header) => Json(header),
        Err(e) => Json(e.to_string()),
    }
}

async fn login(
    headers: HeaderMap,
    uri: Uri,
    State(state): State<PlayerState>,
) -> impl IntoResponse {
    let scheme = extract_scheme(&headers, &uri);
    let host = extract_host(&headers, &uri);
    let redirect_url = format!("{scheme}://{host}/callback");
    match state.oauth.login(redirect_url).await {
        Ok((authorization_url, code_verifier)) => {
            let mut guard = state.code_verifier.lock().await;
            *guard = Some(code_verifier);
            Redirect::to(authorization_url.as_str())
        }
        Err(e) => {
            tracing::error!(%host, %e, "Failed to login");
            Redirect::to("/")
        }
    }
}

async fn callback(
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<CallbackParameters>,
    State(state): State<PlayerState>,
) -> impl IntoResponse {
    let mut guard = state.code_verifier.lock().await;

    match guard.take() {
        Some(code_verifier) => {
            let scheme = extract_scheme(&headers, &uri);
            let host = extract_host(&headers, &uri);
            let redirect_url = format!("{scheme}://{host}/callback");
            if let Err(e) = state
                .oauth
                .authorize(code_verifier, params.code, redirect_url)
                .await
            {
                tracing::error!(%host, %e, "Failed to authorize");
            }
        }
        None => {
            let host = extract_host(&headers, &uri);
            tracing::error!(%host, "Missing code verifier");
        }
    }

    Redirect::to("/")
}

fn extract_scheme(headers: &HeaderMap, uri: &Uri) -> String {
    forwarded_value(headers, "proto")
        .or_else(|| {
            headers
                .get("X-Forwarded-Proto")
                .and_then(|v| v.to_str().ok())
        })
        .map(str::to_owned)
        .or_else(|| uri.scheme_str().map(str::to_owned))
        .unwrap_or_else(|| "http".to_owned())
}

fn extract_host(headers: &HeaderMap, uri: &Uri) -> String {
    forwarded_value(headers, "host")
        .or_else(|| {
            headers
                .get("X-Forwarded-Host")
                .and_then(|v| v.to_str().ok())
        })
        .or_else(|| headers.get(header::HOST).and_then(|v| v.to_str().ok()))
        .map(str::to_owned)
        .or_else(|| uri.authority().map(|a| a.as_str().to_owned()))
        .unwrap_or_else(|| "localhost".to_owned())
}

fn forwarded_value<'a>(headers: &'a HeaderMap, field: &str) -> Option<&'a str> {
    let forwarded = headers.get(header::FORWARDED)?.to_str().ok()?;
    let first = forwarded.split(',').next()?;
    first.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(field)
            .then(|| value.trim().trim_matches('"'))
    })
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../public/index.html"))
}

async fn not_found() -> Html<&'static str> {
    Html(include_str!("../public/404.html"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Uri;

    fn make_headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(
                axum::http::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                value.parse().unwrap(),
            );
        }
        headers
    }

    fn empty_uri() -> Uri {
        "/".parse().unwrap()
    }

    #[test]
    fn extract_scheme_defaults_to_http() {
        let scheme = extract_scheme(&HeaderMap::new(), &empty_uri());
        assert_eq!(scheme, "http");
    }

    #[test]
    fn extract_scheme_from_x_forwarded_proto() {
        let headers = make_headers(&[("X-Forwarded-Proto", "https")]);
        let scheme = extract_scheme(&headers, &empty_uri());
        assert_eq!(scheme, "https");
    }

    #[test]
    fn extract_scheme_from_forwarded_header() {
        let headers =
            make_headers(&[("Forwarded", "host=192.0.2.60;proto=https;by=203.0.113.43")]);
        let scheme = extract_scheme(&headers, &empty_uri());
        assert_eq!(scheme, "https");
    }

    #[test]
    fn extract_scheme_forwarded_takes_precedence_over_x_forwarded_proto() {
        let headers = make_headers(&[
            ("Forwarded", "proto=ftp"),
            ("X-Forwarded-Proto", "https"),
        ]);
        let scheme = extract_scheme(&headers, &empty_uri());
        assert_eq!(scheme, "ftp");
    }

    #[test]
    fn extract_scheme_from_uri() {
        let uri: Uri = "https://example.com/path".parse().unwrap();
        let scheme = extract_scheme(&HeaderMap::new(), &uri);
        assert_eq!(scheme, "https");
    }

    #[test]
    fn extract_host_defaults_to_localhost() {
        let host = extract_host(&HeaderMap::new(), &empty_uri());
        assert_eq!(host, "localhost");
    }

    #[test]
    fn extract_host_from_host_header() {
        let headers = make_headers(&[("Host", "example.com:8080")]);
        let host = extract_host(&headers, &empty_uri());
        assert_eq!(host, "example.com:8080");
    }

    #[test]
    fn extract_host_from_x_forwarded_host() {
        let headers = make_headers(&[("X-Forwarded-Host", "proxy.example.com")]);
        let host = extract_host(&headers, &empty_uri());
        assert_eq!(host, "proxy.example.com");
    }

    #[test]
    fn extract_host_x_forwarded_takes_precedence_over_host_header() {
        let headers = make_headers(&[
            ("X-Forwarded-Host", "proxy.example.com"),
            ("Host", "origin.example.com"),
        ]);
        let host = extract_host(&headers, &empty_uri());
        assert_eq!(host, "proxy.example.com");
    }

    #[test]
    fn extract_host_from_forwarded_header() {
        let headers =
            make_headers(&[("Forwarded", "host=192.0.2.60;proto=http;by=203.0.113.43")]);
        let host = extract_host(&headers, &empty_uri());
        assert_eq!(host, "192.0.2.60");
    }

    #[test]
    fn extract_host_forwarded_takes_precedence_over_x_forwarded_host() {
        let headers = make_headers(&[
            ("Forwarded", "host=forwarded.example.com"),
            ("X-Forwarded-Host", "x-forwarded.example.com"),
        ]);
        let host = extract_host(&headers, &empty_uri());
        assert_eq!(host, "forwarded.example.com");
    }

    #[test]
    fn extract_host_from_uri_authority() {
        let uri: Uri = "https://example.com:9000/path".parse().unwrap();
        let host = extract_host(&HeaderMap::new(), &uri);
        assert_eq!(host, "example.com:9000");
    }
}
