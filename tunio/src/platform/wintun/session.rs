use crate::platform::wintun::handle::HandleWrapper;
use bytes::BufMut;
use log::error;
use std::io;
use std::io::{Read, Write};
use std::sync::Arc;
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_NO_MORE_ITEMS, HANDLE};
use wintun_sys::{DWORD, WINTUN_ADAPTER_HANDLE, WINTUN_SESSION_HANDLE};

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
    ) -> io::Result<Self> {
        let session_handle = unsafe { wintun.WintunStartSession(adapter_handle, capacity) };

        if session_handle.is_null() {
            let err = io::Error::last_os_error();
            error!("Failed to create session: {err}");
            return Err(err);
        }

        Ok(Self {
            handle: HandleWrapper(session_handle),
            wintun,
        })
    }

    pub fn read_event(&self) -> HANDLE {
        HANDLE(unsafe { self.wintun.WintunGetReadWaitEvent(self.handle.0) as isize })
    }
}

impl Read for Session {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        match PacketReader::read(self.handle.clone(), self.wintun.clone()) {
            Ok(packet) => {
                let packet_slice = packet.as_slice();
                buf.put(packet_slice);
                Ok(packet_slice.len())
            }
            Err(err) => {
                if err.raw_os_error().unwrap() == ERROR_NO_MORE_ITEMS.0 as _ {
                    Err(io::Error::from(io::ErrorKind::WouldBlock))
                } else {
                    Err(err)
                }
            }
        }
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
            let err = io::Error::last_os_error();
            if err.raw_os_error().unwrap() == ERROR_BUFFER_OVERFLOW.0 as _ {
                Err(io::Error::from(io::ErrorKind::WouldBlock))
            } else {
                Err(err)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            self.wintun.WintunEndSession(self.handle.0);
        }
    }
}
