use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Context, Error as AnyError};
use oauth2::TokenType;
use oauth2::{
    AuthUrl,
    AuthorizationCode,
    ClientId,
    CsrfToken,
    PkceCodeChallenge,
    PkceCodeVerifier,
    RedirectUrl,
    RefreshToken,
    Scope,
    TokenResponse,
    TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use tokio::sync::Mutex;
use oauth2::url::Url;

const AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPES: &[&str] = &[
    "Files.ReadWrite.All",
    "offline_access", // this scope is required for refresh token
];

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Me {
    id: String,
    display_name: String,
    email: String,
}

#[derive(Clone)]
pub struct ODriveSession {
    inner: Arc<Mutex<Inner>>,
    http_client: reqwest::Client,
}

struct Inner {
    client: BasicClient,
    token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<u64>,
    pkce_verifier: Option<PkceCodeVerifier>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ODrivePersist {
    token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<u64>,
}

impl ODriveSession {
    pub fn new(
        http_client: reqwest::Client,
        client_id: &str,
        redirect_url: &str,
        refresh_token: Option<String>,
        persist: Option<ODrivePersist>,
    ) -> Self {
        let client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            None,
            AuthUrl::new(AUTH_URL.to_string()).unwrap(),
            Some(TokenUrl::new(TOKEN_URL.to_string()).unwrap())
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url.to_string()).unwrap());

        log::info!("OAuth2 client initialized");

        let mut inner = Inner {
            client,
            token: None,
            refresh_token,
            expires_at: None,
            pkce_verifier: None,
        };

        if let Some(persist) = persist {
            inner.token = persist.token;
            inner.refresh_token = persist.refresh_token;
            inner.expires_at = persist.expires_at;
        }
        
        ODriveSession {
            inner: Arc::new(Mutex::new(inner)),
            http_client,
        }
    }

    pub async fn initiate_auth(&self) -> Url {
        log::info!("Initiating authentication");
        let mut guard = self.inner.lock().await;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        guard.pkce_verifier = Some(pkce_verifier);

        let (auth_url, _csrf_token) = guard.client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(SCOPES.iter().map(|s| Scope::new(s.to_string())))
            .set_pkce_challenge(pkce_challenge)
            .url();

        auth_url
    }

    pub async fn auth(&self, authorization_code: &str) -> Result<(), AnyError> {
        log::info!("Authenticating with authorization code");
        let mut guard = self.inner.lock().await;
        let pkce_verifier = guard.pkce_verifier.take().context("PKCE verifier not found")?;

        let token_result = guard.client
            .exchange_code(AuthorizationCode::new(authorization_code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;

        guard.update_tokens(&token_result)?;
        Ok(())
    }

    pub async fn refresh(&self) -> Result<(), AnyError> {
        log::info!("Refreshing token");
        let mut guard = self.inner.lock().await;
        let refresh_token = guard.refresh_token.clone()
            .map(|s| RefreshToken::new(s))
            .context("Refresh token not found")?;
        let token_result = guard.client
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await?;

        guard.update_tokens(&token_result)?;
        Ok(())
    }

    pub async fn me(&self) -> Result<Option<Me>, AnyError> {
        let token = match self.access_token().await {
            Some(t) => t,
            None => return Ok(None),
        };
        let resp = self.http_client.get("https://graph.microsoft.com/v1.0/me")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        Ok(Some(resp.json::<Me>().await?))
    }

    pub async fn access_token(&self) -> Option<String> {
        self.inner.lock().await.token.clone()
    }

    pub async fn to_persist(&self) -> Result<ODrivePersist, AnyError> {
        let guard = self.inner.lock().await;
        Ok(ODrivePersist {
            token: guard.token.clone(),
            refresh_token: guard.refresh_token.clone(),
            expires_at: guard.expires_at,
        })
    }
}

impl Inner {
    fn update_tokens<TR, TT>(self: &mut Self, token_result: &TR) -> Result<(), std::time::SystemTimeError>
    where
        TR: TokenResponse<TT>,
        TT: TokenType,
    {
        self.token = Some(token_result.access_token().secret().clone());
        self.refresh_token = token_result.refresh_token().map(|t| t.secret().clone());
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u64;
        self.expires_at = token_result.expires_in().map(|d| d.as_secs() + now);
        Ok(())
    }
}
