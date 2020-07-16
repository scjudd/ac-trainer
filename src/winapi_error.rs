use std::error;
use std::fmt;

use crate::winapi;

/// Get the last Windows API error that occurred
pub fn last() -> Error {
    let number = unsafe { winapi::GetLastError() };
    let message = message(number);
    Error { number, message }
}

/// Retrieve the error message associated with a given error number
fn message(number: winapi::DWORD) -> String {
    unsafe {
        let flags = winapi::FORMAT_MESSAGE_FROM_SYSTEM
            | winapi::FORMAT_MESSAGE_ALLOCATE_BUFFER
            | winapi::FORMAT_MESSAGE_IGNORE_INSERTS;

        let lang_id = winapi::MAKELANGID(winapi::LANG_NEUTRAL, winapi::SUBLANG_DEFAULT) as u32;

        let mut winapi_allocated_buffer = std::ptr::null_mut();

        let size = winapi::FormatMessageA(
            flags,
            std::ptr::null(),
            number,
            lang_id,
            &mut winapi_allocated_buffer as *mut _ as winapi::LPSTR,
            0,
            std::ptr::null_mut(),
        ) as usize;

        let copied = std::slice::from_raw_parts(winapi_allocated_buffer as *const u8, size)
            .iter()
            .copied()
            .collect();

        winapi::LocalFree(winapi_allocated_buffer);

        String::from_utf8(copied).expect("invalid utf8 data in winapi error message")
    }
}

#[derive(Debug)]
pub struct Error {
    pub number: winapi::DWORD,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {}
