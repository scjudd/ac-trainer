//! Safe abstractions over the Windows API for interacting with remote processes

use crate::winapi;
use crate::winapi_error;

pub type Pid = winapi::DWORD;
pub type Handle = winapi::HANDLE;
pub type Address = u32;

/// Read a type from the memory of a remote process
pub trait Read {
    fn read(handle: Handle, addr: Address) -> Result<Self, String>
    where
        Self: Sized;
}

impl Read for u32 {
    fn read(handle: Handle, addr: Address) -> Result<u32, String> {
        let raw = read(handle, addr, std::mem::size_of::<u32>())?;
        unsafe { Ok(*(raw.as_ptr() as *const u32)) }
    }
}

impl Read for i32 {
    fn read(handle: Handle, addr: Address) -> Result<i32, String> {
        let raw = read(handle, addr, std::mem::size_of::<i32>())?;
        unsafe { Ok(*(raw.as_ptr() as *const i32)) }
    }
}

impl Read for f32 {
    fn read(handle: Handle, addr: Address) -> Result<f32, String> {
        let raw = read(handle, addr, std::mem::size_of::<f32>())?;
        unsafe { Ok(*(raw.as_ptr() as *const f32)) }
    }
}

/// Write a type to the memory of a remote process
pub trait Write {
    fn write(&self, handle: Handle, addr: Address) -> Result<(), String>;
}

impl Write for f32 {
    fn write(&self, handle: Handle, addr: Address) -> Result<(), String> {
        let raw: [u8; 4] = unsafe { std::mem::transmute(*self) };
        write(handle, addr, &raw[..])
    }
}

/// Find the first process having the given name and return its PID
pub fn find(name: &str) -> Option<Pid> {
    unsafe {
        let handle = winapi::CreateToolhelp32Snapshot(winapi::TH32CS_SNAPPROCESS, 0);
        let mut proc: winapi::PROCESSENTRY32 = std::mem::zeroed();
        proc.dwSize = std::mem::size_of::<winapi::PROCESSENTRY32>() as u32;

        let mut ok = winapi::Process32First(handle, &mut proc);

        loop {
            if ok == 0 {
                let err = winapi_error::last();
                if err.number == winapi::ERROR_NO_MORE_FILES {
                    return None;
                }
                panic!(
                    "unexpected Process32(First|Next) error: {}",
                    err.to_string()
                );
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
            let err = winapi_error::last();
            return Err(format!("OpenProcess error: {}", err.to_string()));
        }
        Ok(handle)
    }
}

/// Close the given process handle
pub fn close(handle: Handle) -> Result<(), String> {
    unsafe {
        if winapi::CloseHandle(handle) == 0 {
            let err = winapi_error::last();
            return Err(format!("CloseHandle error: {}", err.to_string()));
        }
        Ok(())
    }
}

/// Check that a given process is still alive
pub fn still_active(handle: Handle) -> Result<bool, String> {
    unsafe {
        let mut exit_code: winapi::DWORD = 0;
        if winapi::GetExitCodeProcess(handle, &mut exit_code as winapi::LPDWORD) == 0 {
            let err = winapi_error::last();
            return Err(format!("GetExitCodeProcess error: {}", err.to_string()));
        }
        Ok(exit_code == winapi::STILL_ACTIVE)
    }
}

/// Read memory from a remote process
pub fn read(handle: Handle, addr: Address, size: usize) -> Result<Vec<u8>, String> {
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
            let err = winapi_error::last();
            return Err(format!("ReadProcessMemory error: {}", err.to_string()));
        }

        // Because we are directly writing to the Vec's internal buffer, we have to manually update
        // its length.
        data.set_len(read);
    }

    Ok(data)
}

/// Write memory to a remote process
pub fn write(handle: Handle, addr: Address, data: &[u8]) -> Result<(), String> {
    let mut written: winapi::SIZE_T = 0;

    unsafe {
        let ok = winapi::WriteProcessMemory(
            handle,
            addr as winapi::LPVOID,
            data as *const _ as winapi::LPCVOID,
            data.len() as winapi::SIZE_T,
            &mut written as *mut winapi::SIZE_T,
        );

        if ok == 0 {
            let err = winapi_error::last();
            return Err(format!("WriteProcessMemory error: {}", err.to_string()));
        }
    }

    Ok(())
}

/// Write memory to a remote process, making sure to handle memory protection setting/resetting
pub fn write_protected(handle: Handle, addr: Address, data: &[u8]) -> Result<(), String> {
    let mut old_protection: winapi::DWORD = 0;

    let ok = unsafe {
        winapi::VirtualProtectEx(
            handle,
            addr as winapi::LPVOID,
            data.len(),
            winapi::PAGE_EXECUTE_READWRITE,
            &mut old_protection as *mut _ as winapi::PDWORD,
        )
    };

    if ok == 0 {
        let err = winapi_error::last();
        return Err(format!("VirtualProtectEx error: {}", err.to_string()));
    }

    write(handle, addr, data)?;

    let ok = unsafe {
        winapi::VirtualProtectEx(
            handle,
            addr as winapi::LPVOID,
            data.len(),
            old_protection,
            &mut old_protection as *mut _ as winapi::PDWORD,
        )
    };

    if ok == 0 {
        let err = winapi_error::last();
        return Err(format!("VirtualProtectEx error: {}", err.to_string()));
    }

    Ok(())
}

/// Allocate memory in a remote process
pub fn alloc_ex(handle: Handle, len: usize) -> Result<Address, String> {
    let addr = unsafe {
        winapi::VirtualAllocEx(
            handle,
            std::ptr::null_mut(),
            len,
            winapi::MEM_COMMIT | winapi::MEM_RESERVE,
            winapi::PAGE_EXECUTE_READWRITE,
        )
    };

    if addr == std::ptr::null_mut() {
        let err = winapi_error::last();
        return Err(format!("VirtualAllocEx error: {}", err.to_string()));
    }

    Ok(addr as Address)
}
