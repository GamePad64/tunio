use super::session::Session;
use crate::platform::wintun::event::SafeEvent;
use crate::traits::QueueT;
use bytes::{Bytes, BytesMut};
use crossbeam_channel::TryRecvError;
use log::warn;
use std::cmp::min;
use std::io::{ErrorKind, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};
use std::{io, thread};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use windows::{
    Win32::System::Threading::{WaitForMultipleObjects, WAIT_OBJECT_0},
    Win32::System::WindowsProgramming::INFINITE,
};
use wintun_sys::WINTUN_MAX_IP_PACKET_SIZE;

impl QueueT for Queue {}

pub struct Queue {
    session: Arc<Mutex<Session>>,

    // Reader
    shutdown_event: Arc<SafeEvent>,

    reader_thread: Option<thread::JoinHandle<()>>,
    packet_rx: crossbeam_channel::Receiver<Bytes>,

    reader_wakers_tx: crossbeam_channel::Sender<Waker>,

    // Writer
    write_status_tx: crossbeam_channel::Sender<io::Result<usize>>,
    write_status_rx: crossbeam_channel::Receiver<io::Result<usize>>,
    packet_writer_thread: Option<tokio::task::JoinHandle<()>>,
}

const WAIT_OBJECT_1: u32 = WAIT_OBJECT_0 + 1;

impl Queue {
    pub(crate) fn new(session: Session) -> Self {
        let session = Arc::new(Mutex::new(session));
        let shutdown_event = Arc::new(SafeEvent::new());

        let inner_session = session.clone();
        let inner_shutdown_event = shutdown_event.clone();

        let (packet_tx, packet_rx) = crossbeam_channel::bounded(16);
        let (reader_wakers_tx, reader_wakers_rx) = crossbeam_channel::unbounded();

        let reader_thread = Some(thread::spawn(move || {
            Self::reader_thread(
                inner_session,
                inner_shutdown_event,
                packet_tx,
                reader_wakers_rx,
            )
        }));

        let (write_status_tx, write_status_rx) = crossbeam_channel::bounded(1);

        Queue {
            session,
            shutdown_event,
            packet_rx,
            reader_thread,
            reader_wakers_tx,
            write_status_tx,
            write_status_rx,
            packet_writer_thread: None,
        }
    }

    fn reader_thread(
        session: Arc<Mutex<Session>>,
        cmd_event: Arc<SafeEvent>,
        packet_tx: crossbeam_channel::Sender<Bytes>,
        wakers_rx: crossbeam_channel::Receiver<Waker>,
    ) {
        let read_event = session.lock().unwrap().read_event();
        let mut buffer = BytesMut::new(); // TODO: use with_capacity with full ring capacity

        'reader: loop {
            buffer.resize(WINTUN_MAX_IP_PACKET_SIZE as _, 0);
            let res = session.lock().unwrap().read(&mut buffer);
            match res {
                Ok(packet_len) => {
                    buffer.truncate(packet_len);
                    packet_tx
                        .send(buffer.split().freeze())
                        .expect("Queue object is ok");

                    // TODO: use single value channel or protected variable
                    if let Some(waker) = wakers_rx.try_iter().last() {
                        waker.wake();
                    }
                }
                Err(err) => {
                    if err.kind() == ErrorKind::WouldBlock {
                        let result = unsafe {
                            WaitForMultipleObjects(&[cmd_event.0, read_event], false, INFINITE)
                        };
                        match result {
                            // Command
                            WAIT_OBJECT_0 => break 'reader,
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
    }
}

impl Read for Queue {
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
                Ok(bytes_to_copy)
            }
        }
    }
}

impl Write for Queue {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.session.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.session.lock().unwrap().flush()
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        // Set reader thread to stop eventually
        self.shutdown_event.set_event();
        // Join thread
        let _ = self.reader_thread.take().unwrap().join();
    }
}

impl AsyncRead for Queue {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
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

impl AsyncWrite for Queue {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let buffer = Bytes::copy_from_slice(buf);

        let inner_session = self.session.clone();
        let inner_write_status_tx = self.write_status_tx.clone();
        let waker = cx.waker().clone();

        if let Ok(result) = self.write_status_rx.try_recv() {
            Poll::Ready(result)
        } else {
            self.get_mut().packet_writer_thread = Some(tokio::task::spawn_blocking(move || {
                let result = inner_session.lock().unwrap().write(&*buffer);

                let _ = inner_write_status_tx.send(result);
                waker.wake();
            }));
            Poll::Pending
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Not implemented by driver
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}
