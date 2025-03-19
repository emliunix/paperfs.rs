use axum::{extract::Query, response::Redirect, routing::{get, post}, Router};
use http::StatusCode;
use serde::Deserialize;

use crate::{odrive::{ODriveSession, ODriveState}, types::OneDriveArgs, utils::LogError};

// Struct to receive the query parameters
#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn login(session: ODriveSession) -> Redirect {
    let url = session.initiate_auth().await;
    // use 303
    Redirect::to(url.as_str())
}

async fn callback<CB: Fn(ODriveState)>(session: ODriveSession, cb: CB, query: CallbackQuery) -> (StatusCode, String) {
    match session.auth(&query.code).await {
        Ok(_) => {
            (cb)(session.state().await);
            log::info!("Authentication successful");
            (StatusCode::OK, "success".to_string())
        },
        Err(e) => {
            log::error!("Authentication failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        },
    }
}

pub fn onedrive_api_router<CB: Fn(ODriveState) + Clone + Send + Sync + 'static>(args: &OneDriveArgs, exposed_url: &String, state: Option<ODriveState>, cb: CB) -> Router {
    let session = ODriveSession::new(
        reqwest::Client::new(),
        &args.client_id,
        args.client_secret.as_ref().map(|s| s.as_str()).unwrap_or(""),
        format!("{}/api/v1/onedrive/callback", exposed_url).as_str(),
        state,
    ).log_err("failed to construct onedrive session");
    Router::new()
        .route("/login", post({
            let session = session.clone();
            || async move { login(session).await } 
        }))
        .route("/callback", get({
            let session = session.clone();
            |Query::<CallbackQuery>(query)| async move {
                callback(session, cb, query).await
            }
        }))
}