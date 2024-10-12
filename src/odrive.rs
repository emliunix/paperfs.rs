
// input: clientId, secrets
// stores client
// Arc<Mutex<>> guarded for inter thread access

use std::sync::Arc;

use msal_rs::{ClientCredential, ConfidentialClient};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ODriveSession<'a> {
    inner: Arc<Mutex<Inner<'a>>>,
}

struct Inner<'a> {
    client: ConfidentialClient,
    scopes: &'a[&'a str],
    token: Option<String>,
    expires_in: Option<u64>,
}

impl<'a> ODriveSession<'a> {
    pub async fn new(
        secret: String,
        authority: &str,
        client_id: &str,
        scopes: &'a[&'a str]
    ) -> Self {
        let client_credential = ClientCredential::from_secret(secret);
        let client = ConfidentialClient::new(client_id, authority, client_credential)
            .await.unwrap();
        log::info!("onedrive auth success");
        
        ODriveSession {
            inner: Arc::new(Mutex::new(Inner {
                client,
                scopes,
                token: None,
                expires_in: None,
            }))
        }
    }
    
    pub async fn update(&self) {
        let mut guard = self.inner.lock().await;
        let inner = &mut *guard;
        inner.client.acquire_token_silent(scopes)
        let resp = inner.client.acquire_token_silent(inner.scopes)
            .await.unwrap();
        (*inner).token = resp.access_token;
        (*inner).expires_in = resp.expires_in;
        log::debug!("expires in {:?}", inner.expires_in);
    }

    pub async fn access_token(&self) -> Option<String> {
        self.inner.lock().await.token.clone()
    }
}

    // From certificate:
    // let client_credential = ClientCredential::from_certificate(
    //     include_bytes!("path/server.pem").to_vec(),
    //     "thumbprint".to_string(),
    // );
