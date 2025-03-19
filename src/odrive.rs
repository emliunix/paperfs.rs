use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Context, Error as AnyError};
use oauth2::*;
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse, BasicTokenType};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use oauth2::url::Url;

const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const SCOPES: &[&str] = &[
    "Files.Read",
    "Files.ReadWrite",
    "offline_access", // this scope is required for refresh token
    "openid", // for id_token
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
    client: OpenIDClient,
    // BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<u64>,
    pkce_verifier: Option<PkceCodeVerifier>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ODriveState {
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIDFields {
    pub id_token: Option<String>,
}

impl ExtraTokenFields for OpenIDFields {}

type OpenIDTokenResponse = StandardTokenResponse<OpenIDFields, BasicTokenType>;

type OpenIDClient = Client<
    BasicErrorResponse,
    OpenIDTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

impl ODriveSession {
    pub fn new(
        http_client: reqwest::Client,
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        state: Option<ODriveState>,
    ) -> Result<Self, anyhow::Error> {
        // BasicClient::new(client_id)
        let client = Client::new(ClientId::new(client_id.to_string()))
            .set_client_secret(ClientSecret::new(client_secret.to_string()))
            .set_auth_uri(AuthUrl::new(AUTH_URL.to_string())?)
            .set_token_uri(TokenUrl::new(TOKEN_URL.to_string())?)
            .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);

        log::info!("OAuth2 client initialized");

        let mut inner = Inner {
            client,
            token: None,
            refresh_token: None,
            expires_at: None,
            pkce_verifier: None,
        };

        if let Some(state) = state {
            inner.refresh_token = state.refresh_token;
            inner.expires_at = state.expires_at;
        }
        
        Ok(ODriveSession {
            inner: Arc::new(Mutex::new(inner)),
            http_client,
        })
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
            .request_async(&self.http_client)
            .await?;

        let id_token = token_result.extra_fields().id_token.as_ref().expect("id_token not present");
        log::debug!("id_token: {}", id_token);

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
            .request_async(&self.http_client)
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

    pub async fn state(&self) -> ODriveState {
        let guard = self.inner.lock().await;
        ODriveState {
            refresh_token: guard.refresh_token.clone(),
            expires_at: guard.expires_at.clone(),
        }
    }
}

impl Inner {
    fn update_tokens(self: &mut Self, token_result: &OpenIDTokenResponse) -> Result<(), std::time::SystemTimeError> {
        self.token = Some(token_result.access_token().secret().clone());
        self.refresh_token = token_result.refresh_token().map(|t| t.secret().clone());
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u64;
        self.expires_at = token_result.expires_in().map(|d| d.as_secs() + now);
        Ok(())
    }
}
