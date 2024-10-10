use std::mem;

use opendal::raw::{oio as oio, LayeredAccess, OpRead, RpRead};
use opendal::raw::{Access, Layer, OpWrite, RpWrite};
use opendal::Result;

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
    type BlockingReader = A::BlockingReader;
    type BlockingWriter = A::BlockingWriter;
    type BlockingLister = A::BlockingLister;

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

    fn blocking_read(
        &self,
        path: &str,
        args: OpRead
    ) -> Result<(RpRead, Self::BlockingReader)> {
        self.access.blocking_read(path, args)
    }

    fn blocking_write(
        &self,
        path: &str,
        args: OpWrite
    ) -> Result<(RpWrite, Self::BlockingWriter)> {
        self.access.blocking_write(path, args)
    }

    fn blocking_list(
        &self,
        path: &str,
        args: opendal::raw::OpList
    ) -> Result<(opendal::raw::RpList, Self::BlockingLister)> {
        self.access.blocking_list(path, args)
    }
}

pub struct BufferedWriter<W> {
    inner: W,
    buffer: Vec<u8>,
}

impl<W: oio::Write> oio::Write for BufferedWriter<W> {
    async fn write(&mut self, bs: opendal::Buffer) -> Result<usize> {
        let len = bs.len();
        log::debug!("buffer {} bytes", len);
        self.buffer.put(bs);
        Ok(len)
    }

    async fn close(&mut self) -> Result<()> {
        log::debug!("write {} bytes", self.buffer.len());
        self.inner.write(mem::replace(&mut self.buffer, Vec::new()).into()).await?;
        self.inner.close().await
    }

    async fn abort(&mut self) -> Result<()> {
        self.inner.abort().await
    }
}

pub struct BlockingBufferedWriter<W> {
    inner: W,
    buffer: Vec<u8>,
}

impl<W: oio::BlockingWrite> oio::BlockingWrite for BlockingBufferedWriter<W> {
    fn write(&mut self, bs: opendal::Buffer) -> Result<usize> {
        let len = bs.len();
        log::debug!("blocking buffer {} bytes", len);
        self.buffer.put(bs);
        Ok(len)
    }

    fn close(&mut self) -> Result<()> {
        log::debug!("blocking write {} bytes", self.buffer.len());
        self.inner.write(mem::replace(&mut self.buffer, Vec::new()).into())?;
        self.inner.close()
    }
}