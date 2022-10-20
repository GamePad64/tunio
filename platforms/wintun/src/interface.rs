use super::queue::SessionQueueT;
use super::wrappers::{Adapter, Session};
use super::PlatformIfConfig;
use super::Queue;
use crate::Driver;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::sync::Arc;
use tunio_core::config::{IfConfig, Layer};
use tunio_core::traits::InterfaceT;
use tunio_core::Error;
use windows::core::GUID;
use windows::Win32::NetworkManagement::IpHelper::ConvertInterfaceLuidToIndex;
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;

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
            &params.name,
            &params.platform.description,
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

    fn handle(&self) -> netconfig::Interface {
        let mut index = 0;
        let luid = NET_LUID_LH {
            Value: self.adapter.luid(),
        };

        unsafe {
            ConvertInterfaceLuidToIndex(&luid, &mut index).unwrap();
        }

        netconfig::Interface::try_from_index(index).unwrap()
    }
}

impl<Q: SessionQueueT> CommonInterface<Q> {
    pub(crate) fn inner_queue_mut(&mut self) -> io::Result<&mut Q> {
        match &mut self.queue {
            Some(queue) => Ok(queue),
            None => Err(ErrorKind::BrokenPipe.into()),
        }
    }
}

pub type Interface = CommonInterface<Queue>;

impl Read for Interface {
    delegate::delegate! {
        to self.inner_queue_mut()? {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
        }
    }
}

impl Write for Interface {
    delegate::delegate! {
        to self.inner_queue_mut()? {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}
