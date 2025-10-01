use crate::console::Screen;
use crate::spotify;
use crate::token::Client;
use axum::extract::{Form, Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, serve};
use axum_extra::extract::Host;
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

async fn login(Host(host): Host, State(state): State<PlayerState>) -> impl IntoResponse {
    let redirect_url = format!("http://{host}/callback");
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
    Query(params): Query<CallbackParameters>,
    Host(host): Host,
    State(state): State<PlayerState>,
) -> impl IntoResponse {
    let mut guard = state.code_verifier.lock().await;

    match guard.take() {
        Some(code_verifier) => {
            let redirect_url = format!("http://{host}/callback");
            if let Err(e) = state
                .oauth
                .authorize(code_verifier, params.code, redirect_url)
                .await
            {
                tracing::error!(%host, %e, "Failed to authorize");
            }
        }
        None => {
            tracing::error!(%host, "Missing code verifier");
        }
    }

    Redirect::to("/")
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../public/index.html"))
}

async fn not_found() -> Html<&'static str> {
    Html(include_str!("../public/404.html"))
}
