use bytes::{BufMut, Bytes, BytesMut};
use crossbeam_channel::TryRecvError;
use log::warn;
use std::cmp::min;
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::{io, thread};
#[cfg(feature = "async-tokio")]
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;
use windows::{
    Win32::Foundation::{ERROR_NO_MORE_ITEMS, HANDLE},
    Win32::System::Threading::{WaitForMultipleObjects, WAIT_OBJECT_0},
    Win32::System::WindowsProgramming::INFINITE,
};

use crate::platform::wintun::event::SafeEvent;
use crate::platform::wintun::handle::HandleWrapper;
use crate::traits::QueueT;
use wintun_sys::{DWORD, WINTUN_SESSION_HANDLE};

impl QueueT for WinTunStream {}

pub struct WinTunStream {
    session_handle: HandleWrapper<WINTUN_SESSION_HANDLE>,

    wintun: Arc<wintun_sys::wintun>,

    // Reader
    cmd_event: Arc<SafeEvent>,

    reader_thread: Option<thread::JoinHandle<()>>,
    packet_rx: crossbeam_channel::Receiver<Bytes>,

    reader_wakers_tx: crossbeam_channel::Sender<Waker>,

    // Writer
    write_status_tx: crossbeam_channel::Sender<std::io::Result<usize>>,
    write_status_rx: crossbeam_channel::Receiver<std::io::Result<usize>>,
    #[cfg(feature = "async-tokio")]
    packet_writer_thread: Option<tokio::task::JoinHandle<()>>,
}

const WAIT_OBJECT_1: u32 = WAIT_OBJECT_0 + 1;

impl WinTunStream {
    pub fn new(
        handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
        wintun: Arc<wintun_sys::wintun>,
    ) -> Self {
        let cmd_event = Arc::new(SafeEvent::new());

        let inner_handle = handle.clone();
        let inner_wintun = wintun.clone();
        let inner_cmd_event = cmd_event.clone();

        let (packet_tx, packet_rx) = crossbeam_channel::bounded(16);
        let (reader_wakers_tx, reader_wakers_rx) = crossbeam_channel::unbounded();

        let reader_thread = Some(thread::spawn(move || {
            Self::reader_thread(
                inner_wintun,
                inner_handle,
                inner_cmd_event,
                packet_tx,
                reader_wakers_rx,
            )
        }));

        let (write_status_tx, write_status_rx) = crossbeam_channel::bounded(1);

        WinTunStream {
            session_handle: handle,
            wintun,
            cmd_event,
            packet_rx,
            reader_thread,
            reader_wakers_tx,
            write_status_tx,
            write_status_rx,
            #[cfg(feature = "async-tokio")]
            packet_writer_thread: None,
        }
    }

    fn reader_thread(
        wintun: Arc<wintun_sys::wintun>,
        handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
        cmd_event: Arc<SafeEvent>,
        packet_tx: crossbeam_channel::Sender<Bytes>,
        wakers_rx: crossbeam_channel::Receiver<Waker>,
    ) {
        let read_event = HANDLE(unsafe { wintun.WintunGetReadWaitEvent(handle.0) as isize });
        let mut buffer = BytesMut::new();

        let mut shutdown = false;

        loop {
            if shutdown {
                break;
            }

            let mut packet_len: DWORD = 0;
            let packet = unsafe { wintun.WintunReceivePacket(handle.0, &mut packet_len) };

            if !packet.is_null() {
                unsafe {
                    let packet_slice = std::slice::from_raw_parts(packet, packet_len as usize);
                    buffer.put(packet_slice);
                    wintun.WintunReleaseReceivePacket(handle.0, packet)
                }
                packet_tx
                    .send(buffer.split().freeze())
                    .expect("Stream object is ok");
                while let Ok(waker) = wakers_rx.try_recv() {
                    waker.wake();
                }
            } else {
                let err = io::Error::last_os_error();
                if err.raw_os_error().unwrap() == ERROR_NO_MORE_ITEMS.0 as _ {
                    let result = unsafe {
                        WaitForMultipleObjects(&[cmd_event.0, read_event], false, INFINITE)
                    };
                    match result {
                        // Command
                        WAIT_OBJECT_0 => shutdown = true,
                        // Ready for read
                        WAIT_OBJECT_1 => continue,

                        e => {
                            panic!("Unexpected event result: {e:?}");
                        }
                    }
                }
            }
        }
    }

    fn shutdown_reader(&mut self) {
        self.cmd_event.set_event();
        let _ = self.reader_thread.take().unwrap().join();
    }

    fn do_write(
        buf: &[u8],
        wintun: Arc<wintun_sys::wintun>,
        session_handle: HandleWrapper<WINTUN_SESSION_HANDLE>,
    ) -> io::Result<usize> {
        let packet = unsafe { wintun.WintunAllocateSendPacket(session_handle.0, buf.len() as _) };
        if !packet.is_null() {
            // Copy buffer to allocated packet
            unsafe {
                packet.copy_from_nonoverlapping(buf.as_ptr(), buf.len());
                wintun.WintunSendPacket(session_handle.0, packet);
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
}

impl Read for WinTunStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.packet_rx.try_recv() {
            Err(TryRecvError::Empty) => Err(io::Error::from(io::ErrorKind::WouldBlock)),
            Err(TryRecvError::Disconnected) => Ok(0),
            Ok(message) => {
                let bytes_to_copy = min(buf.len(), message.len());
                if bytes_to_copy < buf.len() {
                    warn!("Data is truncated: {} > {}", buf.len(), bytes_to_copy);
                }
                buf.copy_from_slice(&message[..bytes_to_copy]);
                Ok(message.len())
            }
        }
    }
}

impl Write for WinTunStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Self::do_write(buf, self.wintun.clone(), self.session_handle.clone())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for WinTunStream {
    fn drop(&mut self) {
        self.shutdown_reader();
        unsafe {
            self.wintun.WintunEndSession(self.session_handle.0);
        }
    }
}

#[cfg(feature = "async-tokio")]
impl AsyncRead for WinTunStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let self_mut = self.get_mut();
        let mut b = vec![0; buf.remaining()];

        match self_mut.read(b.as_mut_slice()) {
            Ok(n) => {
                buf.put_slice(&b[..n]);
                Poll::Ready(Ok(()))
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    let _ = self_mut.reader_wakers_tx.send(cx.waker().clone());
                    Poll::Pending
                } else {
                    Poll::Ready(Err(e))
                }
            }
        }
    }
}

#[cfg(feature = "async-tokio")]
impl AsyncWrite for WinTunStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let buffer = Bytes::copy_from_slice(buf);

        let inner_handle = HandleWrapper(self.session_handle.0);
        let inner_wintun = self.wintun.clone();
        let inner_write_status_tx = self.write_status_tx.clone();
        let waker = cx.waker().clone();

        if let Ok(result) = self.write_status_rx.try_recv() {
            Poll::Ready(result)
        } else {
            self.get_mut().packet_writer_thread = Some(tokio::task::spawn_blocking(move || {
                let inner_handle = inner_handle;

                let result = Self::do_write(&*buffer, inner_wintun.clone(), inner_handle.clone());

                let _ = inner_write_status_tx.send(result);
                waker.wake();
            }));
            Poll::Pending
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
