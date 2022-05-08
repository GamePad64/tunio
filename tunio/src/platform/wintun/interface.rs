use super::wrappers::Adapter;
use super::wrappers::Session;
use super::PlatformIfConfig;
use super::Queue;
use crate::config::{IfConfig, Layer};
use crate::traits::{AsyncQueueT, InterfaceT, QueueT};
use crate::Error;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use windows::core::GUID;
use windows::Win32::NetworkManagement::IpHelper::ConvertInterfaceLuidToIndex;
use wintun_sys;

pub struct Interface {
    wintun: Arc<wintun_sys::wintun>,
    adapter: Arc<Adapter>,
    queue: Option<Queue>,
    config: IfConfig<PlatformIfConfig>,
}

impl QueueT for Interface {}
impl AsyncQueueT for Interface {}

impl InterfaceT for Interface {
    fn up(&mut self) -> Result<(), Error> {
        let session = Session::new(
            self.adapter.clone(),
            self.wintun.clone(),
            self.config.platform.capacity,
        )?;

        self.queue = Some(Queue::new(session));

        Ok(())
    }

    fn down(&mut self) -> Result<(), Error> {
        if self.queue.is_some() {
            let _ = self.queue.take();
            Ok(())
        } else {
            Err(Error::InterfaceStateInvalid)
        }
    }

    fn handle(&self) -> netconfig::InterfaceHandle {
        let mut index = 0;
        unsafe {
            ConvertInterfaceLuidToIndex(&self.adapter.luid(), &mut index).unwrap();
        }

        netconfig::InterfaceHandle::try_from_index(index).unwrap()
    }
}

impl Interface {
    pub(crate) fn new(
        wintun: Arc<wintun_sys::wintun>,
        params: IfConfig<PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let _ = Session::validate_capacity(params.platform.capacity);
        if params.layer == Layer::L2 {
            return Err(Error::LayerUnsupported(params.layer));
        }

        let guid = GUID::new().unwrap();
        let adapter = Arc::new(Adapter::new(
            guid,
            &*params.name,
            &*params.platform.description,
            wintun.clone(),
        )?);

        Ok(Self {
            wintun,
            adapter,
            queue: None,
            config: params,
        })
    }
}

impl Read for Interface {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.queue {
            Some(queue) => queue.read(buf),
            None => Err(std::io::Error::from(ErrorKind::BrokenPipe)),
        }
    }
}

impl Write for Interface {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.queue {
            Some(queue) => queue.write(buf),
            None => Err(std::io::Error::from(ErrorKind::BrokenPipe)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.queue {
            Some(queue) => queue.flush(),
            None => Err(std::io::Error::from(ErrorKind::BrokenPipe)),
        }
    }
}

impl AsyncRead for Interface {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_read(cx, buf),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }
}

impl AsyncWrite for Interface {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_write(cx, buf),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_flush(cx),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match &mut self.queue {
            Some(queue) => Pin::new(queue).poll_shutdown(cx),
            None => Poll::Ready(Err(std::io::Error::from(ErrorKind::BrokenPipe))),
        }
    }
}
