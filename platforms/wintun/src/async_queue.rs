use super::event::SafeEvent;
use super::wrappers::Session;
use crate::queue::SessionQueueT;
use futures::{AsyncRead, AsyncWrite};
use std::future::Future;
use std::io::{self, Read, Write};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::{
    Win32::Foundation::HANDLE, Win32::Foundation::WAIT_ABANDONED_0,
    Win32::Foundation::WAIT_OBJECT_0, Win32::System::Threading::WaitForMultipleObjects,
    Win32::System::WindowsProgramming::INFINITE,
};

enum WaitingStopReason {
    Shutdown,
    Ready,
}

enum ReadState {
    Waiting(Option<async_task::Task<WaitingStopReason>>),
    Idle,
    Closed,
}

pub struct AsyncQueue {
    session: Session,

    read_state: ReadState,
    shutdown_event: Arc<SafeEvent>,
}

impl SessionQueueT for AsyncQueue {
    fn new(session: Session) -> Self {
        Self {
            session,

            read_state: ReadState::Idle,

            // Manual reset, because we use this event once and it must fire on all threads
            shutdown_event: Arc::new(SafeEvent::new(true, false)),
        }
    }
}

impl Drop for AsyncQueue {
    fn drop(&mut self) {
        self.shutdown_event.set_event();
    }
}

fn wait_for_read(read_event: HANDLE, shutdown_event: Arc<SafeEvent>) -> WaitingStopReason {
    const WAIT_OBJECT_1: WIN32_ERROR = WIN32_ERROR(WAIT_OBJECT_0.0 + 1);
    const WAIT_ABANDONED_1: WIN32_ERROR = WIN32_ERROR(WAIT_ABANDONED_0.0 + 1);

    match unsafe { WaitForMultipleObjects(&[shutdown_event.handle(), read_event], false, INFINITE) }
    {
        // Shutdown
        WAIT_OBJECT_0 | WAIT_ABANDONED_0 => WaitingStopReason::Shutdown,
        // Ready for read
        WAIT_OBJECT_1 => WaitingStopReason::Ready,
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
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match &mut self.read_state {
                ReadState::Waiting(task) => {
                    let mut task = task.take().unwrap();

                    self.read_state = match Pin::new(&mut task).poll(cx) {
                        Poll::Ready(WaitingStopReason::Shutdown) => ReadState::Closed,
                        Poll::Ready(WaitingStopReason::Ready) => ReadState::Idle,
                        Poll::Pending => ReadState::Waiting(Some(task)),
                    };

                    if let ReadState::Waiting(..) = self.read_state {
                        return Poll::Pending;
                    }
                }
                ReadState::Idle => match self.session.read(buf) {
                    Ok(n) => return Poll::Ready(Ok(n)),
                    Err(e) => {
                        if e.kind() == io::ErrorKind::WouldBlock {
                            let read_event = self.session.read_event();
                            let inner_shutdown_event = self.shutdown_event.clone();

                            self.read_state =
                                ReadState::Waiting(Some(blocking::unblock(move || {
                                    wait_for_read(read_event, inner_shutdown_event)
                                })));
                        } else {
                            return Poll::Ready(Err(e));
                        }
                    }
                },
                ReadState::Closed => return Poll::Ready(Ok(0)),
            }
        }
    }
}

impl AsyncWrite for AsyncQueue {
    // Write to wintun is already nonblocking
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(self.session.write(buf))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Not implemented by driver
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Not implemented by driver
        Poll::Ready(Ok(()))
    }
}
