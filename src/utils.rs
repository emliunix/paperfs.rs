use std::fmt::Display;

pub trait LogError {
    fn log_err(self, ctx: &'static str);
}

impl<T, E> LogError for Result<T, E> where E: Display {
    fn log_err(self, ctx: &'static str) {
        if let Err(e) = &self {
            log::error!("{}: {}", ctx, e);
        }
    }
}