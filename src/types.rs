
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Plain error: {0}")]
    PlainError(String),
}

pub(crate) fn plain_error<S: ToString>(msg: S) -> impl FnOnce() -> AppError {
    move || { AppError::PlainError(msg.to_string()) }
}


#[derive(Debug, Clone, Default)]
pub struct OneDriveArgs {
    pub onedrive_root: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}