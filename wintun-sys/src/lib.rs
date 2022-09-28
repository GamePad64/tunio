#![allow(non_snake_case, non_camel_case_types)]
#![cfg(target_os = "windows")]

use windows::core::GUID;
use windows::core::PCWSTR as LPCWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH as NET_LUID;
type DWORD = core::ffi::c_ulong;
type BOOL = core::ffi::c_int;
type BYTE = core::ffi::c_uchar;
type DWORD64 = core::ffi::c_ulonglong;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
