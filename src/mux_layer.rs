use std::fmt::Debug;
use std::future::Future;
use std::ops::DerefMut;
use std::sync::Arc;

use futures::lock::Mutex;
use opendal::raw::oio::BlockingList;
use opendal::raw::oio::List;
use opendal::raw::*;
use opendal::ErrorKind;
use opendal::Result;

/// Hopped it can function as a multiplexer of accessors
/// but turns out it's hard to take care of all possible semantic differences
/// eg. memory doesn't support create_dir
pub struct MuxLayer<A, F> {
    f: F,
    a: A,
}

impl<A, F> MuxLayer<A, F> {
    pub fn new(a: A, f: F) -> Self {
        MuxLayer { a, f }
    }
}

pub struct MuxAccess<A, B, F> {
    is_a: F,
    a: A,
    b: B,
}

impl<A, B, F> Debug for MuxAccess<A, B, F> where 
    A: Debug,
    B: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MuxAccess").field("a", &self.a).field("b", &self.b).finish()
    }
}

impl <A, B, F> MuxAccess<A, B, F> {
    fn new(a: A, b: B, f: F) -> Self {
        MuxAccess { a, b, is_a: f }
    }
}

impl<A, AF, F, B: Access> Layer<B> for MuxLayer<AF, F>
where
    AF: Fn() -> A,
    A: Access,
    F: (Fn(&str) -> bool) + 'static + Send + Sync + Unpin + Clone,
    MuxAccess<A, B, F>: Access
{
    type LayeredAccess = MuxAccess<A, B, F>;

    fn layer(&self, inner: B) -> Self::LayeredAccess {
        MuxAccess::new((self.a)(), inner, self.f.clone())
    }
}

impl<A, B, F> Access for MuxAccess<A, B, F>
where 
    A: Access,
    B: Access,
    F: (Fn(&str) -> bool) + 'static + Send + Sync + Unpin + Clone,
{
    type Reader = oio::Reader;
    type Writer = oio::Writer;
    type Lister = oio::Lister;
    type Deleter = oio::Deleter;
    type BlockingReader = ();
    type BlockingWriter = ();
    type BlockingLister = ();
    type BlockingDeleter = ();

    fn info(&self) -> Arc<AccessorInfo> {
        self.b.info()
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        if (self.is_a)(path) {
            let (rp, read) = self.a.read(path, args).await?;
            Ok((rp, Box::new(read)))
        } else {
            let (rp, read) = self.b.read(path, args).await?;
            Ok((rp, Box::new(read)))
        }
    }

    async fn write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        if (self.is_a)(path) {
            let (rp, write) = self.a.write(path, args).await?;
            Ok((rp, Box::new(write)))
        } else {
            let (rp, write) = self.b.write(path, args).await?;
            Ok((rp, Box::new(write)))
        }
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Lister)> {
        log::info!("listing {}", path);
        let (_, list_a) = self.a.list(path, args.clone()).await?;
        let (rp, list_b) = self.b.list(path, args).await?;
        Ok((rp, Box::new(ConcatList::new(list_a, list_b))))
    }

    async fn delete(&self) -> Result<(RpDelete, Self::Deleter)> {
        if (self.is_a)("") {
            let (rp, deleter) = self.a.delete().await?;
            Ok((rp, Box::new(deleter)))
        } else {
            let (rp, deleter) = self.b.delete().await?;
            Ok((rp, Box::new(deleter)))
        }
    }

    async fn stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        log::debug!("stat {}", path);
        if (self.is_a)(path) {
            self.a.stat(path, args).await
        } else {
            self.b.stat(path, args).await
        }
    }

    async fn create_dir(
            &self,
            path: &str,
            args: OpCreateDir,
        ) -> Result<RpCreateDir> {
        log::debug!("create_dir B {}", path);
        self.b.create_dir(path, args).await
    }

    fn blocking_read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::BlockingReader)> {
        Err(opendal::Error::new(ErrorKind::Unsupported, "unsupported"))
    }

    fn blocking_write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::BlockingWriter)> {
        Err(opendal::Error::new(ErrorKind::Unsupported, "unsupported"))
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingLister)> {
        Err(opendal::Error::new(ErrorKind::Unsupported, "unsupported"))
    }

    fn blocking_delete(&self) -> Result<(RpDelete, Self::BlockingDeleter)> {
        Err(opendal::Error::new(ErrorKind::Unsupported, "unsupported"))
    }
}

struct ConcatList<A, B> {
    inner: Arc<Mutex<ConcatList_<A, B>>>,
}

struct ConcatList_<A, B> {
    a: Option<A>,
    b: Option<B>,
}

impl<A, B> ConcatList<A, B> {
    fn new(a: A, b: B) -> Self {
        ConcatList {
            inner: Arc::new(Mutex::new(ConcatList_{a: Some(a), b: Some(b)})),
        }
    }
}

impl<A: oio::List, B: oio::List> oio::List for ConcatList<A, B> {
    fn next(&mut self) -> impl Future<Output = Result<Option<oio::Entry>>> + MaybeSend {
        let self_ = self.inner.clone();
        async move {
            log::info!("listing");
            let mut guard = self_.lock().await;
            if let Some(a) = &mut guard.a {
                log::info!("listing A");
                if let Some(entry) = a.next().await? {
                    log::info!("A entry: {:?}", entry);
                    return Ok(Some(entry))
                }
                (*guard).a = None;
            }
            if let Some(b) = &mut guard.b {
                log::info!("listing B");
                if let Some(entry) = b.next().await? {
                    log::info!("B entry: {:?}", entry);
                    return Ok(Some(entry))
                }
                (*guard).b = None;
            }
            log::info!("listing finished");
            Ok(None)
        }
    }
}
