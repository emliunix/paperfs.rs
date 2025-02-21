use axum::{extract::{Query, State}, response::Redirect, routing::{get, post}, Router};
use http::StatusCode;
use serde::Deserialize;

use crate::{odrive::ODriveSession, types::AppEvents};

async fn login<A>(State((session, app)): State<(ODriveSession, A)>) -> Redirect {
    let url = session.initiate_auth().await;
    // use 303
    Redirect::to(url.as_str())
}

// Struct to receive the query parameters
#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn callback<A: AppEvents>(State((session, app)): State<(ODriveSession, A)>, Query(query): Query<CallbackQuery>) -> (StatusCode, String) {
    match session.auth(&query.code).await {
        Ok(_) => {
            app.on_token_change();
            log::info!("Authentication successful");
            (StatusCode::OK, "success".to_string())
        },
        Err(e) => {
            log::error!("Authentication failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        },
    }
}

pub fn onedrive_api_router<A: AppEvents + Clone + Send + Sync + 'static>(session: ODriveSession, app: A) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/callback", get(callback))
        .with_state((session, app))
}