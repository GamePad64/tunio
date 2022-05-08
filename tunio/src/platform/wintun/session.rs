use super::handle::HandleWrapper;
use crate::Error;
use bytes::BufMut;
use log::error;
use std::io;
use std::io::{Read, Write};
use std::sync::Arc;
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_NO_MORE_ITEMS, HANDLE, WIN32_ERROR};
use wintun_sys::{
    DWORD, WINTUN_ADAPTER_HANDLE, WINTUN_MAX_RING_CAPACITY, WINTUN_MIN_RING_CAPACITY,
    WINTUN_SESSION_HANDLE,
};

struct PacketReader {
    handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
    wintun: Arc<wintun_sys::wintun>,

    ptr: *const u8,
    len: usize,
}

impl PacketReader {
    pub fn read(
        handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
        wintun: Arc<wintun_sys::wintun>,
    ) -> io::Result<Self> {
        let mut len: DWORD = 0;
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

impl Drop for PacketReader {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                self.wintun
                    .WintunReleaseReceivePacket(self.handle.0, self.ptr);
            }
        }
    }
}

pub(crate) struct Session {
    handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
    wintun: Arc<wintun_sys::wintun>,
}

impl Session {
    pub fn new(
        adapter_handle: WINTUN_ADAPTER_HANDLE,
        wintun: Arc<wintun_sys::wintun>,
        capacity: u32,
    ) -> Result<Self, Error> {
        let _ = Self::validate_capacity(capacity)?;

        let session_handle = unsafe { wintun.WintunStartSession(adapter_handle, capacity) };

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

    pub fn read_event(&self) -> HANDLE {
        HANDLE(unsafe { self.wintun.WintunGetReadWaitEvent(self.handle.0) as isize })
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

    fn map_error(err: io::Error, win32_error: WIN32_ERROR) -> io::Error {
        if let Some(os_error) = err.raw_os_error() {
            if os_error == win32_error.0 as _ {
                return io::Error::from(io::ErrorKind::WouldBlock);
            }
        }
        err
    }
}

impl Read for Session {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let packet = PacketReader::read(self.handle.clone(), self.wintun.clone())
            .map_err(|e| Self::map_error(e, ERROR_NO_MORE_ITEMS))?;

        let packet_slice = packet.as_slice();
        buf.put(packet_slice);
        Ok(packet_slice.len())
    }
}

impl Write for Session {
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
            Err(Self::map_error(
                io::Error::last_os_error(),
                ERROR_BUFFER_OVERFLOW,
            ))
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
