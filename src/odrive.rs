use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use thiserror::{Error as ThisError};
use tokio::fs::{File, read_to_string};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::{Context, Error as AnyError};
use oauth2::*;
use oauth2::basic::{BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenType};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use oauth2::url::Url;

use crate::utils::{AsyncHook, LogError, log_and_go};

const APP_DATA_PATH: &str = "app_data.json";
const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const SCOPES: &[&str] = &[
    "Files.Read",
    "Files.ReadWrite",
    "offline_access", // this scope is required for refresh token
    "openid", // for id_token
];

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
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
    states: BTreeMap<String, PkceCodeVerifier>,
    callbacks: Vec<Box<dyn AsyncHook<ODriveState>>>,
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
        client_id: String,
        client_secret: Option<String>,
        redirect_url: String,
    ) -> Result<Self, anyhow::Error> {
        // BasicClient::new(client_id)
        let mut client = Client::new(ClientId::new(client_id))
            .set_auth_uri(AuthUrl::new(AUTH_URL.to_string())?)
            .set_token_uri(TokenUrl::new(TOKEN_URL.to_string())?)
            .set_redirect_uri(RedirectUrl::new(redirect_url)?);
        if let Some(secret) = client_secret {
            client = client.set_client_secret(ClientSecret::new(secret));
        }

        Ok(ODriveSession {
            inner: Arc::new(Mutex::new(Inner {
                client,
                token: None,
                refresh_token: None,
                expires_at: None,
                states: BTreeMap::new(),
                callbacks: Vec::new(),
            })),
            http_client,
        })
    }

    pub async fn initiate_auth(&self) -> Url {
        log::info!("Initiating authentication");
        let mut guard = self.inner.lock().await;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let csrftoken = CsrfToken::new_random();
        log::debug!("PKCE Verifier: {}", pkce_verifier.secret());
        guard.states.insert(csrftoken.secret().clone(), pkce_verifier);

        let (auth_url, _csrf_token) = guard.client
            .authorize_url(move || csrftoken)
            .add_scopes(SCOPES.iter().map(|s| Scope::new(s.to_string())))
            .set_pkce_challenge(pkce_challenge)
            .url();

        auth_url
    }

    pub async fn auth(&self, state: String, authorization_code: String) -> Result<(), AnyError> {
        log::info!("Authenticating with authorization code");
        let mut guard = self.inner.lock().await;
        let verifier = if let Entry::Occupied(entry) = guard.states.entry(state) {
            entry.remove()
        } else {
            return Err(anyhow::anyhow!("auth state not found"));
        };

        let requestor = self.requestor();
        let token_result = guard.client
            .exchange_code(AuthorizationCode::new(authorization_code))
            .set_pkce_verifier(verifier)
            .request_async(&requestor)
            .await?;

        let id_token = token_result.extra_fields().id_token.as_ref().expect("id_token not present");
        log::debug!("id_token: {}", id_token);

        guard.update_tokens(&token_result)?;
        self.call_on_auth().await;
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

        log::info!("Token refreshed successfully");
        guard.update_tokens(&token_result)?;
        self.call_on_auth().await;
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

    pub async fn load_token(&self) -> Result<(), anyhow::Error> {
        // test exists
        if std::path::Path::new(APP_DATA_PATH).exists() {
            let data = read_to_string(APP_DATA_PATH).await?;
            let data: ODriveState = serde_json::from_str(&data).context("failed to deserialize state")?;
            {
                let mut guard = self.inner.lock().await;
                guard.refresh_token = data.refresh_token;
                guard.expires_at = data.expires_at;
            }
            log::info!("Loaded token from {}", APP_DATA_PATH);
            self.refresh().await?;
        }
        Ok(())
    }

    pub fn spawn_token_thread(&self) {
        let session = self.clone();
        tokio::spawn(async move {
            session.token_thread().await;
        });
    }

    pub async fn token_thread(&self) {
        self.on_auth(Box::new(move |state: ODriveState| {
            log_and_go(async move {
                let state_json = serde_json::to_string(&state).context("failed to serialize state")?;
                File::create("app_data.json").await?.write_all(state_json.as_bytes()).await?;
                anyhow::Ok(())
            })
        })).await;
        log_and_go(self.load_token()).await;
        let mut refresh_sec = 300;
        loop {
            log_and_go(self.refresh()).await;
            {
                let guard = self.inner.lock().await;
                if let Some(expires_at) = guard.expires_at {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64;
                    if expires_at > now {
                        refresh_sec = 0.max(expires_at - now - 60); // refresh 1 min before expiry
                    } else {
                        refresh_sec = 0;
                    }
                }
            }
            log::info!("Next token refresh in {} seconds", refresh_sec);
            if refresh_sec > 0 {
                sleep(Duration::from_secs(refresh_sec)).await;
            }
        }
    }

    async fn call_on_auth(&self) {
        let guard = self.inner.lock().await;
        let state = ODriveState { refresh_token: guard.refresh_token.clone(), expires_at: guard.expires_at };
        for cb in guard.callbacks.iter() {
            cb.call(state.clone()).await;
        }
    }

    pub async fn on_auth(&self, cb: Box<dyn AsyncHook<ODriveState>>) {
        let mut guard = self.inner.lock().await;
        guard.callbacks.push(cb);
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
