use pin_project_lite::pin_project;
use std::io;
use std::ops::{Deref, DerefMut};

pin_project! {
    #[derive(Debug)]
    pub(crate) struct Fuse<T, U> {
        #[pin]
        pub t: T,
        pub u: U,
    }
}

impl<T, U> Fuse<T, U> {
    pub(crate) fn new(t: T, u: U) -> Self {
        Self { t, u }
    }
}

impl<T, U> Deref for Fuse<T, U> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.t
    }
}

impl<T, U> DerefMut for Fuse<T, U> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.t
    }
}

cfg_async! {
    use futures_util::io::{AsyncRead, AsyncWrite};
    use std::marker::Unpin;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    impl<T: AsyncRead + Unpin, U> AsyncRead for Fuse<T, U> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            self.project().t.poll_read(cx, buf)
        }
    }

    impl<T: AsyncWrite + Unpin, U> AsyncWrite for Fuse<T, U> {
        fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
            self.project().t.poll_write(cx, buf)
        }
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
            self.project().t.poll_flush(cx)
        }
        fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
            self.project().t.poll_close(cx)
        }
    }
}

cfg_blocking! {
    use std::io::{Read, Write};

    impl<T: Read, U> Read for Fuse<T, U> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.t.read(buf)
        }
    }

    impl<T: Write, U> Write for Fuse<T, U> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.t.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.t.flush()
        }
    }
}
