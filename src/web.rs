use crate::token::Client;
use axum::extract::{Form, Host, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::serve;
use oauth2::PkceCodeVerifier;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch::{Receiver, Sender};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};
use crate::console::Screen;

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
}

impl PlayerState {
    fn new(
        sender: Sender<Option<String>>,
        _receiver: Receiver<Option<String>>,
        oauth: Client,
        screen: Screen,
    ) -> Self {
        Self {
            sender,
            _receiver,
            oauth,
            screen,
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
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address).await?;
    let serve_dir = ServeDir::new("public").not_found_service(ServeFile::new("public/404.html"));
    let app = axum::Router::new()
        .route("/logs", get(logs))
        .route("/play", post(play).put(play))
        .route("/login", get(login))
        .route("/callback", get(callback))
        .fallback_service(serve_dir)
        .with_state(PlayerState::new(sender, receiver, oauth, screen));

    serve(listener, app).await?;

    Ok(())
}

async fn logs(State(state): State<PlayerState>) -> String {
    tracing::info!("logs");
    state.screen.read().join("")
}


async fn play(State(state): State<PlayerState>, Form(input): Form<Input>) -> impl IntoResponse {
    let value = Some(input.uri).filter(|v| !v.is_empty());

    if let Err(e) = state.sender.send(value) {
        tracing::error!(%e, "Failed to set desired state as playing");
    }

    Redirect::to("/")
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
