use super::event::SafeEvent;
use super::wrappers::Session;
use crate::platform::wintun::queue::SessionQueueT;
use futures::{AsyncRead, AsyncWrite};
use parking_lot::Mutex;
use std::io::{self, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::{
    Win32::Foundation::HANDLE, Win32::Foundation::WAIT_ABANDONED_0,
    Win32::Foundation::WAIT_OBJECT_0, Win32::System::Threading::WaitForMultipleObjects,
    Win32::System::WindowsProgramming::INFINITE,
};

pub struct AsyncQueue {
    session: Arc<Mutex<Session>>,

    shutdown_event: Arc<SafeEvent>,
    data_ready: Arc<Mutex<DataReadinessHandler>>,
}

impl SessionQueueT for AsyncQueue {
    fn new(session: Session) -> Self {
        Self {
            session: Arc::new(Mutex::new(session)),
            // Manual reset, because we use this event once and it must fire on all threads
            shutdown_event: Arc::new(SafeEvent::new(true, false)),
            data_ready: Default::default(),
        }
    }
}

impl Drop for AsyncQueue {
    fn drop(&mut self) {
        self.shutdown_event.set_event();
    }
}

const WAIT_OBJECT_1: WIN32_ERROR = WIN32_ERROR(WAIT_OBJECT_0.0 + 1);
const WAIT_ABANDONED_1: WIN32_ERROR = WIN32_ERROR(WAIT_ABANDONED_0.0 + 1);

#[derive(Default)]
struct DataReadinessHandler {
    tokio_wait_thread: Option<tokio::task::JoinHandle<()>>,
    waker: Option<Waker>,
}

fn wait_for_read(
    read_event: HANDLE,
    shutdown_event: Arc<SafeEvent>,
    data_ready: Arc<Mutex<DataReadinessHandler>>,
) {
    let result =
        unsafe { WaitForMultipleObjects(&[shutdown_event.handle(), read_event], false, INFINITE) };
    match result {
        // Shutwown
        WAIT_OBJECT_0 => {}
        // Ready for read
        WAIT_OBJECT_1 => {
            let mut data_ready = data_ready.lock();
            if let Some(waker) = (*data_ready).waker.take() {
                waker.wake()
            }
        }
        // Shutdown event deleted
        WAIT_ABANDONED_0 => {}
        // Read event deleted
        WAIT_ABANDONED_1 => {
            panic!("Read event deleted unexpectedly");
        }

        e => {
            panic!("Unexpected event result: {e:?}");
        }
    }
}

impl AsyncRead for AsyncQueue {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut data_ready = self.data_ready.lock();
        if (*data_ready).waker.is_some() {
            // Waker has not been executed yet, so I/O is still not ready. Replace the waker.
            (*data_ready).waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

        let mut session = self.session.lock();
        match session.read(buf) {
            Ok(n) => {
                // No need to schedule waker
                Poll::Ready(Ok(n))
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    // Schedule waker for execution later
                    (*data_ready).waker = Some(cx.waker().clone());

                    let read_event = session.read_event();
                    let inner_shutdown_event = self.shutdown_event.clone();
                    let inner_data_ready = self.data_ready.clone();

                    (*data_ready).tokio_wait_thread =
                        Some(tokio::task::spawn_blocking(move || {
                            wait_for_read(read_event, inner_shutdown_event, inner_data_ready);
                        }));
                    Poll::Pending
                } else {
                    Poll::Ready(Err(e))
                }
            }
        }
    }
}

impl AsyncWrite for AsyncQueue {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.session.lock().write(buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(err) => {
                if err.kind() != io::ErrorKind::WouldBlock {
                    Poll::Ready(Err(err))
                } else {
                    let waker = cx.waker().clone();
                    let _ = tokio::task::spawn_local(async { waker.wake() });
                    Poll::Pending
                }
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Not implemented by driver
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Not implemented by driver
        Poll::Ready(Ok(()))
    }
}
