use axum::extract::{Form, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::post;
use axum::serve;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::watch::{Receiver, Sender};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Deserialize)]
struct Input {
    uri: String,
}

#[derive(Clone)]
struct PlayerState {
    sender: Sender<Option<String>>,
    _receiver: Receiver<Option<String>>,
}

impl PlayerState {
    fn new(sender: Sender<Option<String>>, _receiver: Receiver<Option<String>>) -> Self {
        Self { sender, _receiver }
    }
}

pub async fn run(
    sender: Sender<Option<String>>,
    receiver: Receiver<Option<String>>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:5853").await?;
    let serve_dir = ServeDir::new("public").not_found_service(ServeFile::new("public/404.html"));
    let app = axum::Router::new()
        .route("/play", post(play).put(play))
        .fallback_service(serve_dir)
        .with_state(PlayerState::new(sender, receiver));

    serve(listener, app).await?;

    Ok(())
}

async fn play(State(state): State<PlayerState>, Form(input): Form<Input>) -> impl IntoResponse {
    let value = Some(input.uri).filter(|v| !v.is_empty());

    if let Err(e) = state.sender.send(value) {
        tracing::error!(%e, "Failed to set desired state as playing");
    }

    Redirect::to("/")
}
