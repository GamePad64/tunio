use super::Adapter;
use super::HandleWrapper;
use bytes::BufMut;
use log::error;
use std::io;
use std::io::{Read, Write};
use std::sync::Arc;
use tunio_core::Error;
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_NO_MORE_ITEMS, HANDLE, WIN32_ERROR};
use wintun_sys::{WINTUN_MAX_RING_CAPACITY, WINTUN_MIN_RING_CAPACITY, WINTUN_SESSION_HANDLE};

struct PacketReader<'a> {
    handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
    wintun: &'a wintun_sys::wintun,

    ptr: *const u8,
    len: usize,
}

impl<'a> PacketReader<'a> {
    pub fn read(
        handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
        wintun: &'a wintun_sys::wintun,
    ) -> io::Result<Self> {
        let mut len: u32 = 0;
        let ptr = unsafe { wintun.WintunReceivePacket(handle.0, &mut len) };

        if !ptr.is_null() {
            Ok(Self {
                handle,
                wintun,
                ptr,
                len: len as _,
            })
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a> Drop for PacketReader<'a> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                self.wintun
                    .WintunReleaseReceivePacket(self.handle.0, self.ptr);
            }
        }
    }
}

pub struct Session {
    handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
    wintun: Arc<wintun_sys::wintun>,
}

impl Session {
    pub fn new(
        adapter: Arc<Adapter>,
        wintun: Arc<wintun_sys::wintun>,
        capacity: u32,
    ) -> Result<Self, Error> {
        let _ = Self::validate_capacity(capacity)?;

        let session_handle = unsafe { wintun.WintunStartSession(adapter.handle(), capacity) };

        if session_handle.is_null() {
            let err = io::Error::last_os_error();
            error!("Failed to create session: {err}");
            return Err(err.into());
        }

        Ok(Self {
            handle: HandleWrapper(session_handle),
            wintun,
        })
    }

    #[allow(dead_code)]
    pub fn read_event(&self) -> HANDLE {
        unsafe { self.wintun.WintunGetReadWaitEvent(self.handle.0) }
    }

    pub fn validate_capacity(capacity: u32) -> Result<(), Error> {
        let range = WINTUN_MIN_RING_CAPACITY..=WINTUN_MAX_RING_CAPACITY;
        if !range.contains(&capacity) || !capacity.is_power_of_two() {
            return Err(Error::InvalidConfigValue {
                name: "capacity".to_string(),
                value: capacity.to_string(),
                reason: format!(
                    "must be power of 2 between {} and {}",
                    WINTUN_MIN_RING_CAPACITY, WINTUN_MAX_RING_CAPACITY
                ),
            });
        }
        Ok(())
    }
}

impl Read for Session {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let packet = PacketReader::read(self.handle.clone(), &self.wintun);
        match packet {
            Ok(packet) => {
                let packet_slice = packet.as_slice();
                buf.put(packet_slice);
                Ok(packet_slice.len())
            }
            Err(e) => match error_eq(&e, ERROR_NO_MORE_ITEMS) {
                true => Err(io::ErrorKind::WouldBlock.into()),
                false => Err(e),
            },
        }
    }
}

impl Write for Session {
    // does not block, as WintunAllocateSendPacket and WintunSendPacket are executed right one ofter another
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let packet = unsafe {
            self.wintun
                .WintunAllocateSendPacket(self.handle.0, buf.len() as _)
        };
        if !packet.is_null() {
            // Copy buffer to allocated packet
            unsafe {
                packet.copy_from_nonoverlapping(buf.as_ptr(), buf.len());
                self.wintun.WintunSendPacket(self.handle.0, packet); // Deallocates packet
            }
            Ok(buf.len())
        } else {
            let e = io::Error::last_os_error();
            match error_eq(&e, ERROR_BUFFER_OVERFLOW) {
                true => panic!("send buffer overflow"),
                false => Err(e),
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            self.wintun.WintunEndSession(self.handle.0);
        }
    }
}

fn error_eq(err: &io::Error, win32_error: WIN32_ERROR) -> bool {
    match err.raw_os_error() {
        None => false,
        Some(os_error) => os_error == win32_error.0 as _,
    }
}
