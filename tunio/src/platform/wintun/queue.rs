use super::event::SafeEvent;
use super::wrappers::Session;
use crate::traits::{AsyncQueueT, QueueT};
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use windows::{
    Win32::System::Threading::{WaitForMultipleObjects, WAIT_ABANDONED_0, WAIT_OBJECT_0},
    Win32::System::WindowsProgramming::INFINITE,
};

impl QueueT for Queue {}
impl AsyncQueueT for Queue {}

#[derive(Default)]
struct DataReadinessHandler {
    wait_thread: Option<tokio::task::JoinHandle<()>>,
    waker: Option<Waker>,
}

pub struct Queue {
    session: Arc<Mutex<Session>>,

    shutdown_event: Arc<SafeEvent>,
    data_ready: Arc<Mutex<DataReadinessHandler>>,
}

const WAIT_OBJECT_1: u32 = WAIT_OBJECT_0 + 1;
const WAIT_ABANDONED_1: u32 = WAIT_ABANDONED_0 + 1;

impl Queue {
    pub(crate) fn new(session: Session) -> Self {
        Self {
            session: Arc::new(Mutex::new(session)),
            // Manual reset, because we use this event once and it must fire on all threads
            shutdown_event: Arc::new(SafeEvent::new(true, false)),
            data_ready: Default::default(),
        }
    }

    fn wait_for_read(
        session: Arc<Mutex<Session>>,
        shutdown_event: Arc<SafeEvent>,
        data_ready: Arc<Mutex<DataReadinessHandler>>,
    ) {
        let read_event = session.lock().unwrap().read_event();
        let result = unsafe {
            WaitForMultipleObjects(&[shutdown_event.handle(), read_event], false, INFINITE)
        };
        match result {
            // Shutwown
            WAIT_OBJECT_0 => {}
            // Ready for read
            WAIT_OBJECT_1 => {
                let mut data_ready = data_ready.lock().unwrap();
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
}

impl Read for Queue {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.session.lock().unwrap().read(buf)
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
        self.shutdown_event.set_event();
    }
}

impl AsyncRead for Queue {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut data_ready = self.data_ready.lock().unwrap();
        if (*data_ready).waker.is_some() {
            // Waker has not been executed yet, so I/O is still not ready. Replace the waker.
            (*data_ready).waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

        match self.session.lock().unwrap().read(buf.initialize_unfilled()) {
            Ok(n) => {
                // No need to schedule waker
                buf.set_filled(n);
                Poll::Ready(Ok(()))
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    // Schedule waker for execution later
                    (*data_ready).waker = Some(cx.waker().clone());

                    let inner_session = self.session.clone();
                    let inner_shutdown_event = self.shutdown_event.clone();
                    let inner_data_ready = self.data_ready.clone();

                    (*data_ready).wait_thread = Some(tokio::task::spawn_blocking(move || {
                        Self::wait_for_read(inner_session, inner_shutdown_event, inner_data_ready);
                    }));
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
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.write(buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(err) => {
                if err.kind() != ErrorKind::WouldBlock {
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

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Not implemented by driver
        Poll::Ready(Ok(()))
    }
}
