use crate::queue::create_device;
use crate::{Driver, PlatformIfConfig};
use delegate::delegate;
use futures::{AsyncRead, AsyncWrite};
use netconfig::sys::InterfaceExt;
use std::io::{self, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use tunio_core::config::IfConfig;
use tunio_core::queue::syncfd::SyncFdQueue;
#[cfg(feature = "tokio")]
use tunio_core::queue::tokiofd::TokioFdQueue;
use tunio_core::queue::FdQueueT;
use tunio_core::traits::{AsyncQueueT, InterfaceT, SyncQueueT};
use tunio_core::Error;

pub struct UtunInterface<Q> {
    name: String,
    queue: Q,
}

impl<Q: FdQueueT> InterfaceT for UtunInterface<Q> {
    type PlatformDriver = Driver;
    type PlatformIfConfig = PlatformIfConfig;

    fn new_up(
        _driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let queue = Q::new(create_device(&params.name, Q::BLOCKING)?);

        Ok(Self {
            name: params.name,
            queue,
        })
    }

    fn new(
        driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let mut interface = Self::new_up(driver, params)?;
        interface.down()?;
        Ok(interface)
    }

    fn up(&mut self) -> Result<(), Error> {
        let handle = self.handle();
        handle.set_up(true)?;
        handle.set_running(true)?;

        Ok(())
    }

    fn down(&mut self) -> Result<(), Error> {
        let handle = self.handle();
        handle.set_up(false)?;
        handle.set_running(false)?;

        Ok(())
    }

    fn handle(&self) -> netconfig::Interface {
        netconfig::Interface::try_from_name(self.name()).unwrap()
    }
}

impl<Q> UtunInterface<Q> {
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub type Interface = UtunInterface<SyncFdQueue>;

impl SyncQueueT for Interface {}

impl<Q: SyncQueueT> Read for UtunInterface<Q> {
    delegate! {
        to self.queue {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>;
        }
    }
}

impl<Q: SyncQueueT> Write for UtunInterface<Q> {
    delegate! {
        to self.queue {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}

#[cfg(feature = "tokio")]
pub type TokioInterface = UtunInterface<TokioFdQueue>;
#[cfg(feature = "tokio")]
impl AsyncQueueT for TokioInterface {}

impl<Q: AsyncQueueT> AsyncRead for UtunInterface<Q> {
    delegate! {
        to Pin::new(&mut self.queue) {
            fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>>;
        }
    }
}

impl<Q: AsyncQueueT> AsyncWrite for UtunInterface<Q> {
    delegate! {
        to Pin::new(&mut self.queue) {
            fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>>;
            fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
            fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
        }
    }
}
