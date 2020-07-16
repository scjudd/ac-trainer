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
///
/// The message is copied into an owned String and returned to the caller. All temporary
/// allocations by the Windows API functions are accounted for and freed before the function
/// returns.
fn message(number: winapi::DWORD) -> String {
    let mut flags = 0;
    flags |= winapi::FORMAT_MESSAGE_FROM_SYSTEM;
    flags |= winapi::FORMAT_MESSAGE_ALLOCATE_BUFFER;
    flags |= winapi::FORMAT_MESSAGE_IGNORE_INSERTS;

    let lang_id = winapi::MAKELANGID(winapi::LANG_NEUTRAL, winapi::SUBLANG_DEFAULT) as u32;

    let mut buf_ptr: *mut _ = std::ptr::null_mut();

    let size = unsafe {
        winapi::FormatMessageA(
            flags,
            std::ptr::null(),
            number,
            lang_id,
            &mut buf_ptr as *mut _ as winapi::LPSTR,
            0,
            std::ptr::null_mut(),
        )
    };

    let message_slice = unsafe {
        std::slice::from_raw_parts(buf_ptr as *const u8, size as usize)
    };

    let message = {
        let bytes = message_slice
            .iter()
            .copied()
            .collect::<Vec<u8>>();

        String::from_utf8(bytes).expect("invalid utf8 data in winapi error message")
    };

    unsafe {
        winapi::LocalFree(buf_ptr as *mut _);
    }

    message
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
