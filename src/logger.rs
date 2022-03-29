use crate::wintun_raw;
use log::{error, info, warn};
use widestring::U16CStr;

pub unsafe extern "C" fn wintun_logger(
    level: wintun_raw::WINTUN_LOGGER_LEVEL,
    _timestamp: wintun_raw::DWORD64,
    message: *const wintun_raw::WCHAR,
) {
    let message = U16CStr::from_ptr_str(message);
    let message_utf8 = message.to_string_lossy();

    match level {
        wintun_raw::WINTUN_LOGGER_LEVEL_WINTUN_LOG_INFO => info!("{message_utf8}"),
        wintun_raw::WINTUN_LOGGER_LEVEL_WINTUN_LOG_WARN => warn!("{message_utf8}"),
        wintun_raw::WINTUN_LOGGER_LEVEL_WINTUN_LOG_ERR => error!("{message_utf8}"),
        _ => error!("[invalid log level: {level}] {message_utf8}"),
    }
}
