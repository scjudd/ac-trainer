#![allow(dead_code)]

use crate::winapi;

pub type Pid = winapi::DWORD;
pub type Handle = winapi::HANDLE;

/// Find the first process having the given name and return its PID
pub fn find(name: &str) -> Option<Pid> {
    unsafe {
        let handle = winapi::CreateToolhelp32Snapshot(winapi::TH32CS_SNAPPROCESS, 0);
        let mut proc: winapi::PROCESSENTRY32 = std::mem::zeroed();
        proc.dwSize = std::mem::size_of::<winapi::PROCESSENTRY32>() as u32;

        let mut ok = winapi::Process32First(handle, &mut proc);

        loop {
            if ok == 0 {
                let errno = winapi::GetLastError();
                if errno == winapi::ERROR_NO_MORE_FILES {
                    return None;
                }
                panic!("unexpected Process32(First|Next) error: {}", errno);
            }

            let exe = std::ffi::CStr::from_ptr(&proc.szExeFile as *const winapi::CHAR);
            if exe.to_str().unwrap() == name {
                return Some(proc.th32ProcessID);
            }

            ok = winapi::Process32Next(handle, &mut proc);
        }
    }
}

/// Open the process identified by the given PID
pub fn open(pid: Pid) -> Result<Handle, String> {
    unsafe {
        let handle = winapi::OpenProcess(winapi::PROCESS_ALL_ACCESS, 0, pid);
        if handle == std::ptr::null_mut() {
            let errno = winapi::GetLastError();
            return Err(format!("OpenProcess error: {}", errno));
        }
        Ok(handle)
    }
}

/// Close the given process handle
pub fn close(handle: Handle) -> Result<(), String> {
    unsafe {
        if winapi::CloseHandle(handle) == 0 {
            let errno = winapi::GetLastError();
            return Err(format!("CloseHandle error: {}", errno));
        }
        Ok(())
    }
}

/// Check that a given process is still alive
pub fn still_active(handle: Handle) -> Result<bool, String> {
    unsafe {
        let mut exit_code: winapi::DWORD = 0;
        if winapi::GetExitCodeProcess(handle, &mut exit_code as winapi::LPDWORD) == 0 {
            let errno = winapi::GetLastError();
            return Err(format!("GetExitCodeProcess error: {}", errno));
        }
        Ok(exit_code == winapi::STILL_ACTIVE)
    }
}

/// Read memory from a remote process
pub fn read(handle: Handle, addr: u32, size: usize) -> Result<Vec<u8>, String> {
    let mut data = Vec::with_capacity(size);
    let mut read: winapi::SIZE_T = 0;

    unsafe {
        let ok = winapi::ReadProcessMemory(
            handle,
            addr as winapi::LPVOID,
            data.as_mut_ptr() as winapi::LPVOID,
            size as winapi::SIZE_T,
            &mut read as *mut winapi::SIZE_T,
        );

        if ok == 0 {
            let errno = winapi::GetLastError();
            return Err(format!("ReadProcessMemory error: {}", errno));
        }

        // Because we are directly writing to the Vec's internal buffer, we have to manually update
        // its length.
        data.set_len(read);
    }

    Ok(data)
}
