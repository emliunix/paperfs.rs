use std::mem;

use opendal::raw::{oio as oio, LayeredAccess, OpRead, RpRead};
use opendal::raw::{Access, Layer, OpWrite, RpWrite};
use opendal::{Metadata, Result};

use bytes::BufMut;

#[derive(Debug, Copy, Clone)]
pub struct BufLayer;

impl Default for BufLayer {
    fn default() -> Self {
        Self { }
    }
}

impl<A: Access> Layer<A> for BufLayer {
    type LayeredAccess = BufAccessor<A>;

    fn layer(&self, access: A) -> Self::LayeredAccess {
        BufAccessor { access }
    }
}

#[derive(Debug)]
pub struct BufAccessor<A> where A: Access {
    access: A,
}

impl<A:Access> LayeredAccess for BufAccessor<A> {
    type Inner = A;
    type Reader = A::Reader;
    type Writer = BufferedWriter<A::Writer>;
    type Lister = A::Lister;
    type Deleter = A::Deleter;

    fn inner(&self) -> &Self::Inner {
        &self.access
    }

    async fn read(
        &self,
        path: &str,
        args: OpRead,
    ) -> Result<(RpRead, Self::Reader)> {
        self.access.read(path, args).await
    }

    async fn write(
        &self,
        path: &str,
        args: OpWrite,
    ) -> Result<(RpWrite, Self::Writer)> {
        let (rp_write, writer) = self.access.write(path, args).await?;
        Ok((rp_write, BufferedWriter { inner: writer, buffer: Vec::new() }))
    }

    async fn list(
        &self,
        path: &str,
        args: opendal::raw::OpList,
    ) -> Result<(opendal::raw::RpList, Self::Lister)> {
        self.access.list(path, args).await
    }

    async fn delete(&self) -> Result<(opendal::raw::RpDelete, Self::Deleter)> {
        self.access.delete().await
    }
}

pub struct BufferedWriter<W> {
    inner: W,
    buffer: Vec<u8>,
}

impl<W: oio::Write> oio::Write for BufferedWriter<W> {
    async fn write(&mut self, bs: opendal::Buffer) -> Result<()> {
        log::debug!("buffer {} bytes", bs.len());
        self.buffer.put(bs);
        Ok(())
    }

    async fn close(&mut self) -> Result<Metadata> {
        log::debug!("write {} bytes", self.buffer.len());
        self.inner.write(mem::replace(&mut self.buffer, Vec::new()).into()).await?;
        self.inner.close().await
    }

    async fn abort(&mut self) -> Result<()> {
        log::debug!("abort");
        self.inner.abort().await
    }
}
