use std::{fmt::{Debug, Display}, future::Future, pin::Pin};

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

pub fn log_and_go<Fut, E>(fut: Fut) -> impl Future<Output=()> where
    Fut: Future<Output = Result<(), E>>,
    E: Display,
{
    async {
        if let Err(e) = fut.await {
            log::error!("silented error: {}", e);
        }
    }
}

pub trait AsyncHook<Arg>: Send + Sync {
    fn call<'a>(
        &'a self,
        arg: Arg,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

    fn clone_box(&self) -> Box<dyn AsyncHook<Arg>>;
}

impl<Arg, F, Fut> AsyncHook<Arg> for F
where
    F: Fn(Arg) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn call<'a>(
        &'a self,
        arg: Arg,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin((self)(arg))
    }
    
    fn clone_box(&self) -> Box<dyn AsyncHook<Arg>> {
        Box::new(self.clone())
    }
}

impl<Arg> Clone for Box<dyn AsyncHook<Arg>> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}