use crate::waiter::Waiter;
use crate::wintun_raw;
use rand::{Fill, Rng};
use std::io::Read;
use std::sync::Arc;
use widestring::U16CString;

pub(crate) struct WinTunDriver {
    wintun: Arc<wintun_raw::wintun>,
    waiter: Arc<Waiter>,
}

pub struct WinTunInterface {
    handle: wintun_raw::WINTUN_ADAPTER_HANDLE,
}

fn encode_utf16(string: &str, max_characters: usize) -> U16CString {
    let utf16 = U16CString::from_str(string).unwrap();
    if utf16.len() >= max_characters {
        panic!("too long")
    } else {
        utf16
    }
}

impl WinTunDriver {
    pub(crate) fn new() -> Self {
        Self {
            wintun: Arc::new(unsafe { wintun_raw::wintun::new("wintun").unwrap() }),
            waiter: Arc::new(Waiter::new()),
        }
    }
}

const MAX_RING_CAPACITY: u32 = 0x400_0000;
const MIN_RING_CAPACITY: u32 = 0x2_0000;
const MAX_POOL: u32 = 256;

impl WinTunInterface {
    pub(crate) fn new(wintun: Arc<wintun_raw::wintun>, name: &str, r#type: &str) -> Self {
        let (name_u16, type_u16) = (
            encode_utf16(name, wintun::MAX_POOL),
            encode_utf16(r#type, wintun::MAX_POOL),
        );
        let mut guid_bytes: [u8; 16] = [0u8; 16];
        rand::thread_rng().fill(&mut guid_bytes);
        let guid = u128::from_ne_bytes(guid_bytes);

        let guid_struct: wintun_raw::GUID = unsafe { std::mem::transmute(guid) };
        let guid_ptr = &guid_struct as *const wintun_raw::GUID;

        let result =
            unsafe { wintun.WintunCreateAdapter(name_u16.as_ptr(), type_u16.as_ptr(), guid_ptr) };

        Self { handle: result }
    }

    // pub(crate) fn send(&self, buf: &[u8]) {
    //     let mut packet = self.session.allocate_send_packet(buf.len() as u16).unwrap();
    //     packet.bytes_mut().copy_from_slice(buf);
    //     self.session.send_packet(packet);
    // }
}
