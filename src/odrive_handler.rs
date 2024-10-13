use axum::{extract::{Query, State}, response::Redirect, routing::{get, post}, Router};
use serde::Deserialize;

use crate::{odrive::ODriveSession, types::AppEvents};


async fn login<A>(State((session, app)): State<(ODriveSession, A)>) -> Redirect {
    let url = session.initiate_auth().await;
    Redirect::temporary(url.as_str())
}

// Struct to receive the query parameters
#[derive(Deserialize)]
struct AuthCallbackQuery {
    code: String,
    state: String,
}

async fn callback<A: AppEvents>(State((session, app)): State<(ODriveSession, A)>, Query(query): Query<AuthCallbackQuery>) -> &'static str {
    app.on_auth(query.code);
    "success"
}

pub fn onedrive_api_router<A: AppEvents + Clone + Send + Sync + 'static>(session: ODriveSession, app: A) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/callback", get(callback))
        .with_state((session, app))
}