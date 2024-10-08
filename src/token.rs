use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::Instant;
use url::Url;

pub struct Client {
    client: BasicClient,
    path: PathBuf,
    token: BasicTokenResponse,
    deadline: Instant,
}

impl Client {
    pub fn new(client_id: String, path: PathBuf) -> Self {
        let client_id = ClientId::new(client_id);

        let auth_url = AuthUrl::new("https://accounts.spotify.com/authorize".to_string())
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new("https://accounts.spotify.com/api/token".to_string())
            .expect("Invalid token endpoint URL");
        let redirect_url =
            RedirectUrl::new("http://localhost:2474".to_string()).expect("Invalid redirect URL");

        let client = BasicClient::new(client_id, None, auth_url, Some(token_url))
            .set_redirect_uri(redirect_url);

        let mut deadline = Instant::now();
        let token = std::fs::read_to_string(&path)
            .ok()
            .and_then(|token| serde_json::from_str(&token).ok())
            .unwrap_or_else(|| {
                let token = authorize(&client).expect("Failed to authorize the client");
                deadline += token.expires_in().unwrap_or_default();
                save(&path, &token).expect("Failed to save the token");
                token
            });

        Self {
            client,
            path,
            token,
            deadline,
        }
    }

    fn refresh(&mut self) {
        if self.deadline < Instant::now() {
            self.token = refresh(&self.client, &self.token).expect("Failed to refresh the token");
            self.deadline = Instant::now() + self.token.expires_in().unwrap_or_default();
            save(&self.path, &self.token).expect("Failed to save the token");
        }
    }

    pub fn authorization(&mut self) -> String {
        self.refresh();

        let secret = self.token.access_token().secret();

        match self.token.token_type().as_ref() {
            "bearer" => format!("Bearer {secret}"),
            token_type => format!("{token_type} {secret}"),
        }
    }
}

pub fn authorize(client: &BasicClient) -> anyhow::Result<BasicTokenResponse> {
    // Create a Proof Key of Code Exchange code verifier and SHA-256 encode it as a code challenge.
    let (code_challenge, code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user-read-private".to_string()))
        .add_scope(Scope::new("user-read-email".to_string()))
        .add_scope(Scope::new("user-read-playback-state".to_string()))
        .add_scope(Scope::new("user-modify-playback-state".to_string()))
        .add_scope(Scope::new("user-read-currently-playing".to_string()))
        .add_scope(Scope::new("streaming".to_string()))
        .add_scope(Scope::new("playlist-read-private".to_string()))
        .set_pkce_challenge(code_challenge)
        .url();

    println!("Open this URL in your browser:\n{authorize_url}\n");

    let code = {
        // A very naive implementation of the redirect server.
        let listener = TcpListener::bind("127.0.0.1:2474")?;

        // The server will terminate itself after collecting the first code.
        let Some(mut stream) = listener.incoming().flatten().next() else {
            panic!("listener terminated without accepting a connection");
        };

        let mut reader = BufReader::new(&stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        let redirect_url = request_line.split_whitespace().nth(1).unwrap();
        let url = Url::parse(&("http://localhost".to_string() + redirect_url))?;

        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, code)| AuthorizationCode::new(code.into_owned()))
            .unwrap();

        let message = "Go back to your terminal :)";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        stream.write_all(response.as_bytes())?;

        code
    };

    let token = client
        .exchange_code(code)
        .set_pkce_verifier(code_verifier)
        .request(http_client)?;

    Ok(token)
}

pub fn save(path: impl AsRef<Path>, token: &BasicTokenResponse) -> anyhow::Result<()> {
    let token = serde_json::to_string(&token)?;

    std::fs::write(path, token)?;

    Ok(())
}

fn refresh(client: &BasicClient, token: &BasicTokenResponse) -> anyhow::Result<BasicTokenResponse> {
    let token = client
        .exchange_refresh_token(token.refresh_token().expect("Missing refresh token"))
        .request(http_client)?;

    Ok(token)
}
