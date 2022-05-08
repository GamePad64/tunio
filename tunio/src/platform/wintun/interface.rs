use super::handle::HandleWrapper;
use super::queue::Queue;
use super::session::Session;
use super::PlatformIfConfig;
use crate::config::IfConfig;
use crate::traits::{AsyncQueueT, InterfaceT, QueueT};
use crate::Error;
use log::error;
use std::io::{ErrorKind, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{io, ptr};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use widestring::U16CString;
use windows::core::GUID;
use windows::Win32::NetworkManagement::IpHelper::{ConvertInterfaceLuidToIndex, NET_LUID_LH};
use wintun_sys;
use wintun_sys::WINTUN_ADAPTER_HANDLE;

const MAX_NAME: usize = 255;

pub struct Interface {
    wintun: Arc<wintun_sys::wintun>,
    handle: HandleWrapper<WINTUN_ADAPTER_HANDLE>,
    queue: Option<Queue>,
    config: IfConfig<PlatformIfConfig>,
}

impl QueueT for Interface {}
impl AsyncQueueT for Interface {}

impl InterfaceT for Interface {
    fn up(&mut self) -> Result<(), Error> {
        let session = Session::new(
            self.handle.0,
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
            ConvertInterfaceLuidToIndex(&self.luid(), &mut index).unwrap();
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

        let [name_u16, description_u16] =
            [&*params.name, &*params.platform.description].map(encode_name);
        let (name_u16, description_u16) = (name_u16?, description_u16?);

        let guid = GUID::new().unwrap();
        let guid = wintun_sys::GUID {
            Data1: guid.data1,
            Data2: guid.data2,
            Data3: guid.data3,
            Data4: guid.data4,
        };

        let adapter_handle = unsafe {
            wintun.WintunCreateAdapter(
                name_u16.as_ptr(),
                description_u16.as_ptr(),
                &guid as *const wintun_sys::GUID,
            )
        };

        if adapter_handle.is_null() {
            let err = io::Error::last_os_error();
            error!("Failed to create adapter: {err}");
            return Err(Error::from(err));
        }

        Ok(Self {
            wintun,
            handle: HandleWrapper(adapter_handle),
            queue: None,
            config: params,
        })
    }

    fn luid(&self) -> NET_LUID_LH {
        let mut luid_buf: wintun_sys::NET_LUID = unsafe { std::mem::zeroed() };
        unsafe {
            self.wintun
                .WintunGetAdapterLUID(self.handle.0, &mut luid_buf as _)
        }
        NET_LUID_LH {
            Value: unsafe { luid_buf.Value },
        }
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        unsafe { self.wintun.WintunCloseAdapter(self.handle.0) };
        self.handle = HandleWrapper(ptr::null_mut());
    }
}

fn encode_name(string: &str) -> Result<U16CString, Error> {
    let result = U16CString::from_str(string).map_err(|_| Error::InterfaceNameUnicodeError)?;
    match result.len() {
        0..=MAX_NAME => Ok(result),
        l => Err(Error::InterfaceNameTooLong(l, MAX_NAME)),
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
