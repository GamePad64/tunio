use get_last_error::Win32Error;
use log::{trace, warn};
use std::sync::mpsc as stdmpsc;
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::mpsc as tokmpsc;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::HANDLE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::synchapi::{CreateEventA, SetEvent, WaitForMultipleObjects};
use winapi::um::winbase::{INFINITE, WAIT_ABANDONED_0, WAIT_FAILED, WAIT_OBJECT_0};

pub(crate) struct UnsafeHandle<T>(pub T);

unsafe impl<T> Send for UnsafeHandle<T> {}

unsafe impl<T> Sync for UnsafeHandle<T> {}

enum Command {
    AddHandle {
        handle: UnsafeHandle<HANDLE>,
        channel: SenderChannelT,
    },
    Shutdown {
        completion_channel: oneshot::Sender<()>,
    },
}

pub struct Waiter {
    cmd_event: HANDLE,
    cmd_channel_tx: stdmpsc::Sender<Command>,

    worker_joinhandle: Option<JoinHandle<()>>,
}

type SenderChannelT = tokmpsc::UnboundedSender<()>;
type ReceiverChannelT = tokmpsc::UnboundedReceiver<()>;

struct WorkerThreadState {
    handles: Vec<HANDLE>,
    channels: Vec<SenderChannelT>,
}

impl WorkerThreadState {
    fn new(interrupt_event: HANDLE) -> Self {
        Self {
            handles: vec![interrupt_event],
            channels: vec![],
        }
    }

    fn add_handle(&mut self, handle: HANDLE, channel: SenderChannelT) -> usize {
        self.handles.push(handle);
        self.channels.push(channel);
        self.channels.len() - 1
    }

    fn remove_handle(&mut self, number: usize) {
        // 0 is first channel
        self.handles.remove(number + 1);
        self.channels.remove(number);
    }

    fn total_handles_dword(&self) -> DWORD {
        self.handles.len() as DWORD
    }
}

impl Waiter {
    pub(crate) fn new() -> Self {
        let cmd_event = unsafe { CreateEventA(std::ptr::null_mut(), 0, 0, std::ptr::null_mut()) };

        let cmd_event_unsafe = UnsafeHandle(cmd_event);
        let (cmd_channel_tx, cmd_channel_rx) = stdmpsc::channel();

        let waiter_join =
            thread::spawn(move || Self::worker_thread(cmd_event_unsafe, cmd_channel_rx));

        Self {
            cmd_event,
            cmd_channel_tx,
            worker_joinhandle: Some(waiter_join),
        }
    }

    fn worker_thread(cmd_event: UnsafeHandle<HANDLE>, cmd_channel_rx: stdmpsc::Receiver<Command>) {
        let mut state = WorkerThreadState::new(cmd_event.0);
        loop {
            trace!("WFMO starting with handles: {:?}", state.handles);
            let result_code = unsafe {
                WaitForMultipleObjects(
                    state.total_handles_dword(),
                    state.handles.as_ptr(),
                    0,
                    INFINITE,
                )
            };
            trace!("WFMO result: {result_code:?}");

            if result_code == WAIT_OBJECT_0 {
                loop {
                    match cmd_channel_rx.try_recv() {
                        Ok(Command::AddHandle {
                            handle: UnsafeHandle(handle),
                            channel,
                        }) => {
                            state.add_handle(handle, channel);
                        }
                        Ok(Command::Shutdown { completion_channel }) => {
                            let _ = unsafe { CloseHandle(state.handles[0]) };
                            let _ = completion_channel.send(());
                        }
                        Err(_) => break,
                    }
                }
            } else if result_code == WAIT_ABANDONED_0 {
                panic!("Interrupt event is abandoned");
            } else if ((WAIT_OBJECT_0 + 1)..(WAIT_OBJECT_0 + state.total_handles_dword()))
                .contains(&result_code)
            {
                let channel_index = (result_code - WAIT_OBJECT_0 - 1) as usize;
                trace!("Fired handle {channel_index}");
                if state.channels[channel_index].send(()).is_err() {
                    state.remove_handle(channel_index);
                }
            } else if ((WAIT_ABANDONED_0 + 1)..(WAIT_ABANDONED_0 + state.total_handles_dword()))
                .contains(&result_code)
            {
                let channel_index = (result_code - WAIT_ABANDONED_0 - 1) as usize;
                warn!("Abandoned handle {channel_index}");
                if state.channels[channel_index].send(()).is_err() {
                    state.remove_handle(channel_index);
                }
            } else if result_code == WAIT_FAILED {
                let err = Win32Error::get_last_error();
                panic!("Wait failed: {err}");
            } else {
                unimplemented!()
            }
        }
    }

    fn interrupt(&self, command: Command) {
        let _ = self.cmd_channel_tx.send(command);
        let _ = unsafe { SetEvent(self.cmd_event) };
    }

    pub(crate) fn handle_channel(&self, handle: HANDLE) -> ReceiverChannelT {
        let (channel_tx, channel_rx) = tokmpsc::unbounded_channel();

        self.interrupt(Command::AddHandle {
            handle: UnsafeHandle(handle),
            channel: channel_tx,
        });

        channel_rx
    }
}

impl Drop for Waiter {
    fn drop(&mut self) {
        let (completion_channel_tx, completion_channel_rx) = oneshot::channel();
        self.interrupt(Command::Shutdown {
            completion_channel: completion_channel_tx,
        });
        let _ = completion_channel_rx.recv();
        let _ = self.worker_joinhandle.take().unwrap().join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::UnboundedReceiver;

    #[tokio::test]
    async fn test_single_event() {
        let event = unsafe { CreateEventA(std::ptr::null_mut(), 0, 0, std::ptr::null_mut()) };

        let waiter = Waiter::new();
        let mut chan = waiter.handle_channel(event);

        assert!(chan.try_recv().is_err());
        unsafe {
            SetEvent(event);
        }
        assert!(chan.recv().await.is_some());
    }

    #[tokio::test]
    async fn test_multi_event() {
        let handles: Vec<HANDLE> = (0..10)
            .map(|_| unsafe { CreateEventA(std::ptr::null_mut(), 0, 0, std::ptr::null_mut()) })
            .collect();

        let waiter = Waiter::new();

        let mut channels: Vec<UnboundedReceiver<()>> = handles
            .iter()
            .map(|h| waiter.handle_channel(h.clone()))
            .collect();

        for channel in channels.iter_mut() {
            assert!(channel.try_recv().is_err());
        }

        for handle in &handles {
            unsafe {
                SetEvent(*handle);
            }
        }

        for channel in channels.iter_mut() {
            assert!(channel.recv().await.is_some());
        }
    }
}
