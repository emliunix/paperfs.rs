use std::fmt::Debug;

pub trait LogError {
    type Output;
    fn log_err(self, ctx: &'static str) -> Self::Output;
}

impl<T, E> LogError for Result<T, E> where E: Debug {
    type Output = T;
    fn log_err(self, ctx: &'static str) -> T {
        if let Err(e) = &self {
            log::error!("{}: {:?}", ctx, e);
        }
        self.unwrap()
    }
}