use log::{error, info, warn};
use widestring::U16CStr;
use windows::core::PCWSTR;
use wintun_sys::{
    WINTUN_LOGGER_LEVEL, WINTUN_LOGGER_LEVEL_WINTUN_LOG_ERR, WINTUN_LOGGER_LEVEL_WINTUN_LOG_INFO,
    WINTUN_LOGGER_LEVEL_WINTUN_LOG_WARN,
};

pub unsafe extern "C" fn wintun_logger(
    level: WINTUN_LOGGER_LEVEL,
    _timestamp: u64,
    message: PCWSTR,
) {
    let message = U16CStr::from_ptr_str(message.as_ptr());
    let message_utf8 = message.to_string_lossy();

    match level {
        WINTUN_LOGGER_LEVEL_WINTUN_LOG_INFO => info!("{message_utf8}"),
        WINTUN_LOGGER_LEVEL_WINTUN_LOG_WARN => warn!("{message_utf8}"),
        WINTUN_LOGGER_LEVEL_WINTUN_LOG_ERR => error!("{message_utf8}"),
        _ => error!("[invalid log level: {level}] {message_utf8}"),
    }
}
