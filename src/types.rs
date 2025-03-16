
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
