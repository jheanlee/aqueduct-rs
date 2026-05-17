/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::io::IoSlice;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct ProxyIO<R, W> {
    reader: R,
    writer: W,
    write_counter: Arc<AtomicI64>,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> ProxyIO<R, W> {
    pub fn new(reader: R, writer: W, write_counter: Arc<AtomicI64>) -> Self {
        Self {
            reader,
            writer,
            write_counter,
        }
    }
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> AsyncRead for ProxyIO<R, W> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.reader).poll_read(cx, buf)
    }
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> AsyncWrite for ProxyIO<R, W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let res = Pin::new(&mut this.writer).poll_write(cx, buf);
        match res {
            Poll::Ready(Ok(size)) => {
                this.write_counter.fetch_add(size as i64, Ordering::Relaxed);
                Poll::Ready(Ok(size))
            }
            result => result,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.writer).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.writer).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let res = Pin::new(&mut this.writer).poll_write_vectored(cx, bufs);
        match res {
            Poll::Ready(Ok(size)) => {
                this.write_counter.fetch_add(size as i64, Ordering::Relaxed);
                Poll::Ready(Ok(size))
            }
            result => result,
        }
    }

    fn is_write_vectored(&self) -> bool {
        self.writer.is_write_vectored()
    }
}
