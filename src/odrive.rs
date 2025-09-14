use std::marker::PhantomData;
use thiserror::{Error as ThisError};
use std::pin::Pin;
use std::future::Future;
use std::error::Error;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Context, Error as AnyError};
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

#[derive(ThisError, Debug)]
enum RequestorError {
    #[error("HTTP Error: {0}")]
    HTTPError(reqwest::Error),
    #[error("Build response Error: {0}")]
    BuildResponseError(http::Error),
}

impl ODriveSession {
    pub fn new(
        http_client: reqwest::Client,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_url: &str,
        state: Option<ODriveState>,
    ) -> Result<Self, anyhow::Error> {
        // BasicClient::new(client_id)
        let mut client = Client::new(ClientId::new(client_id.to_string()))
            .set_auth_uri(AuthUrl::new(AUTH_URL.to_string())?)
            .set_token_uri(TokenUrl::new(TOKEN_URL.to_string())?)
            .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);
        if let Some(secret) = client_secret {
            client = client.set_client_secret(ClientSecret::new(secret.to_string()));
        }

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
        log::debug!("PKCE Verifier: {}", pkce_verifier.secret());
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

        let requestor = self.requestor();
        let token_result = guard.client
            .exchange_code(AuthorizationCode::new(authorization_code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&requestor)
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
        let requestor = self.requestor();
        let token_result = guard.client
            .exchange_refresh_token(&refresh_token)
            .request_async(&requestor)
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

    fn requestor(&self) -> impl Fn(HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, RequestorError>> + Send>> + use<'_> {
        move |request| {
            let http_client = self.http_client.clone();
            Box::pin(async move {
                log::debug!("Making HTTP request: {:?}", request);
                let res = http_client.execute(request.try_into().map_err(RequestorError::HTTPError)?)
                    .await
                    .map_err(RequestorError::HTTPError)?;
                log::debug!("Received HTTP response: {:?}", res);
                let mut builder = http::Response::builder().status(res.status());
                for (name, value) in res.headers().iter() {
                    builder = builder.header(name, value);
                }
                builder
                    .body(res.bytes().await.map_err(RequestorError::HTTPError)?.to_vec())
                    .map_err(RequestorError::BuildResponseError)
            })
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
