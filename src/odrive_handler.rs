use axum::{Router, extract::{Query, State}, response::Redirect, routing::{get, post}};
use http::StatusCode;
use serde::Deserialize;

use crate::odrive::ODriveSession;

// Struct to receive the query parameters
#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn login(State(session): State<ODriveSession>) -> Redirect {
    let url = session.initiate_auth().await;
    // use 303
    Redirect::to(url.as_str())
}

async fn callback(State(session): State<ODriveSession>, Query(query): Query<CallbackQuery>) -> (StatusCode, String) {
    match session.auth(query.state, query.code).await {
        Ok(_) => {
            log::info!("Authentication successful");
            (StatusCode::OK, "success".to_string())
        },
        Err(e) => {
            log::error!("Authentication failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        },
    }
}

pub fn onedrive_api_router(session: ODriveSession) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/callback", get(callback))
        .with_state(session)
}