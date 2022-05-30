use super::queue::SessionQueueT;
use super::wrappers::{Adapter, Session};
use super::PlatformIfConfig;
use super::Queue;
use crate::config::{IfConfig, Layer};
use crate::platform::wintun::Driver;
use crate::traits::InterfaceT;
use crate::Error;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::sync::Arc;
use windows::core::GUID;
use windows::Win32::NetworkManagement::IpHelper::ConvertInterfaceLuidToIndex;
use wintun_sys;

pub struct CommonInterface<Q: SessionQueueT> {
    wintun: Arc<wintun_sys::wintun>,
    adapter: Arc<Adapter>,
    config: IfConfig<PlatformIfConfig>,
    pub(crate) queue: Option<Q>,
}

impl<Q: SessionQueueT> InterfaceT for CommonInterface<Q> {
    type PlatformDriver = Driver;
    type PlatformIfConfig = PlatformIfConfig;

    fn new(
        driver: &mut Self::PlatformDriver,
        params: IfConfig<Self::PlatformIfConfig>,
    ) -> Result<Self, Error> {
        let _ = Session::validate_capacity(params.platform.capacity);
        if params.layer == Layer::L2 {
            return Err(Error::LayerUnsupported(params.layer));
        }

        let wintun = driver.wintun().clone();

        let adapter = Arc::new(Adapter::new(
            GUID::from_u128(params.platform.guid),
            &*params.name,
            &*params.platform.description,
            wintun.clone(),
        )?);

        Ok(Self {
            wintun,
            adapter,
            config: params,
            queue: None,
        })
    }

    fn up(&mut self) -> Result<(), Error> {
        let session = Session::new(
            self.adapter.clone(),
            self.wintun.clone(),
            self.config.platform.capacity,
        )?;
        self.queue = Some(Q::new(session));

        Ok(())
    }

    fn down(&mut self) -> Result<(), Error> {
        let _ = self.queue.take();
        Ok(())
    }

    fn handle(&self) -> netconfig::InterfaceHandle {
        let mut index = 0;
        unsafe {
            ConvertInterfaceLuidToIndex(&self.adapter.luid(), &mut index).unwrap();
        }

        netconfig::InterfaceHandle::try_from_index(index).unwrap()
    }
}

pub type Interface = CommonInterface<Queue>;

impl Read for Interface {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.queue {
            Some(queue) => queue.read(buf),
            None => Err(io::Error::from(ErrorKind::BrokenPipe)),
        }
    }
}

impl Write for Interface {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.queue {
            Some(queue) => queue.write(buf),
            None => Err(io::Error::from(ErrorKind::BrokenPipe)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.queue {
            Some(queue) => queue.flush(),
            None => Err(io::Error::from(ErrorKind::BrokenPipe)),
        }
    }
}
