use oauth2::basic::{BasicClient, BasicTokenResponse, BasicTokenType};
use oauth2::{AccessToken, AuthUrl, AuthorizationCode, ClientId, CsrfToken, EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use url::Url;

#[derive(Clone, Serialize, Deserialize)]
struct CachedToken {
    access_token: AccessToken,
    refresh_token: RefreshToken,
    token_type: BasicTokenType,
    deadline: SystemTime,
}

impl CachedToken {
    fn new(token: BasicTokenResponse, now: SystemTime) -> anyhow::Result<Self> {
        Ok(Self {
            access_token: token.access_token().clone(),
            refresh_token: token
                .refresh_token()
                .ok_or_else(|| anyhow::anyhow!("No refresh token"))?
                .clone(),
            token_type: token.token_type().clone(),
            deadline: now + token.expires_in().unwrap_or_default(),
        })
    }
}

#[derive(Clone)]
pub struct Client {
    client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    http: reqwest::Client,
    path: PathBuf,
    token: Arc<Mutex<Option<CachedToken>>>,
}

impl Client {
    pub fn new(client_id: String, path: PathBuf) -> Self {
        let client_id = ClientId::new(client_id);

        let auth_url = AuthUrl::new("https://accounts.spotify.com/authorize".to_string())
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new("https://accounts.spotify.com/api/token".to_string())
            .expect("Invalid token endpoint URL");

        let client = BasicClient::new(client_id)
            .set_auth_uri(auth_url)
            .set_token_uri(token_url);
        let http = reqwest::Client::new();

        Self {
            client,
            http,
            path,
            token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn authorization(&mut self) -> anyhow::Result<String> {
        let token = self.refresh().await?;
        let secret = token.access_token.secret();

        match token.token_type {
            BasicTokenType::Bearer => Ok(format!("Bearer {secret}")),
            token_type => Err(anyhow::anyhow!(
                "Unsupported token type: {}",
                token_type.as_ref()
            )),
        }
    }

    async fn refresh(&mut self) -> anyhow::Result<CachedToken> {
        let now = SystemTime::now();
        let mut guard = self.token.lock().await;

        let mut token = match guard.as_ref() {
            Some(token) => token.clone(),
            None => load(&self.path).await?,
        };

        if token.deadline <= now {
            let response = self
                .client
                .exchange_refresh_token(&token.refresh_token)
                .request_async(&self.http)
                .await?;

            token = CachedToken::new(response, now)?;
            *guard = Some(token.clone());
            save(&self.path, &token).await?;
        }

        Ok(token)
    }

    pub async fn login(&self, redirect_url: String) -> anyhow::Result<(Url, PkceCodeVerifier)> {
        // Create a Proof Key of Code Exchange code verifier and SHA-256 encode it as a code challenge.
        let (code_challenge, code_verifier) = PkceCodeChallenge::new_random_sha256();
        let redirect_url = RedirectUrl::new(redirect_url)?;

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, _) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user-read-private".to_string()))
            .add_scope(Scope::new("user-read-email".to_string()))
            .add_scope(Scope::new("user-read-playback-state".to_string()))
            .add_scope(Scope::new("user-modify-playback-state".to_string()))
            .add_scope(Scope::new("user-read-currently-playing".to_string()))
            .add_scope(Scope::new("streaming".to_string()))
            .add_scope(Scope::new("playlist-read-private".to_string()))
            .set_pkce_challenge(code_challenge)
            .set_redirect_uri(Cow::Owned(redirect_url))
            .url();

        Ok((authorize_url, code_verifier))
    }

    pub async fn authorize(
        &self,
        code_verifier: PkceCodeVerifier,
        code: String,
        redirect_url: String,
    ) -> anyhow::Result<()> {
        let now = SystemTime::now();
        let code = AuthorizationCode::new(code);
        let response = self
            .client
            .exchange_code(code)
            .set_pkce_verifier(code_verifier)
            .set_redirect_uri(Cow::Owned(RedirectUrl::new(redirect_url)?))
            .request_async(&self.http)
            .await?;
        let token = CachedToken::new(response, now)?;

        save(&self.path, &token).await?;

        let mut guard = self.token.lock().await;
        *guard = Some(token);

        Ok(())
    }
}

async fn save(path: impl AsRef<Path>, token: &CachedToken) -> anyhow::Result<()> {
    let token = serde_json::to_string(token)?;
    tokio::fs::write(path, token).await?;
    Ok(())
}

async fn load(path: impl AsRef<Path>) -> anyhow::Result<CachedToken> {
    let contents = tokio::fs::read_to_string(path).await?;
    let token = serde_json::from_str(&contents)?;
    Ok(token)
}
