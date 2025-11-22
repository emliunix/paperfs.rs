use axum::{Json, Router, extract::{Query, State}, response::Redirect, routing::{get, post}};
use http::StatusCode;
use serde::Deserialize;

use crate::odrive::{Me, ODriveSession};

// Struct to receive the query parameters
#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

#[derive(serde::Serialize)]
struct Response<T> where T: serde::Serialize {
    code: u16,
    msg: String,
    body: T,
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

async fn me(State(session): State<ODriveSession>) -> (StatusCode, Json<Response<Option<Me>>>) {
    match session.me().await {
        Ok(Some(info)) => (StatusCode::OK, Json(Response {
            code: StatusCode::OK.as_u16(),
            msg: "success".to_string(),
            body: Some(info),
        })),
        Ok(None) => (StatusCode::NOT_FOUND, Json(Response {
            code: StatusCode::NOT_FOUND.as_u16(),
            msg: "user info not found".to_string(),
            body: None,
        })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Response {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            msg: format!("error retrieving user info: {}", e),
            body: None,
        })),
    }
}

pub fn onedrive_api_router(session: ODriveSession) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/callback", get(callback))
        .route("/me", get(me))
        .with_state(session)
}