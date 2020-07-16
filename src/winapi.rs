//! The minimal set of bindings for the Windows API
//!
//! This module should define only the needed set of bindings; it is not meant to be exhaustive.
//! Most of the type aliases are borrowed from https://docs.rs/winapi/0.3.8/.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub type c_char = i8;
pub type c_short = i16;
pub type c_ushort = u16;
pub type c_int = i32;
pub type c_long = i32;
pub type c_ulong = u32;
pub enum c_void {}

pub type BOOL = c_int;
pub type CHAR = c_char;
pub type DWORD = c_ulong;
pub type LONG = c_long;
pub type ULONG_PTR = usize;
pub type PDWORD = *mut DWORD;
pub type LPDWORD = *mut DWORD;
pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type SHORT = c_short;
pub type USHORT = c_ushort;
pub type SIZE_T = ULONG_PTR;
pub type HANDLE = *mut c_void;
pub type HLOCAL = HANDLE;
pub type LPPROCESSENTRY32 = *mut PROCESSENTRY32;
pub type LANGID = USHORT;
pub type LPSTR = *mut CHAR;
pub type va_list = *mut c_char;

pub const TH32CS_SNAPPROCESS: DWORD = 0x00000002;

pub const ERROR_NO_MORE_FILES: DWORD = 18;

pub const PROCESS_ALL_ACCESS: DWORD = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0xFFFF;
pub const STANDARD_RIGHTS_REQUIRED: DWORD = 0x000F0000;
pub const SYNCHRONIZE: DWORD = 0x00100000;

pub const STILL_ACTIVE: DWORD = 259;

pub const VK_CAPITAL: c_int = 0x14;

pub const MEM_COMMIT: DWORD = 0x1000;
pub const MEM_RESERVE: DWORD = 0x2000;

pub const PAGE_EXECUTE_READWRITE: DWORD = 0x40;

pub const FORMAT_MESSAGE_FROM_SYSTEM: DWORD = 0x00001000;
pub const FORMAT_MESSAGE_ALLOCATE_BUFFER: DWORD = 0x00000100;
pub const FORMAT_MESSAGE_IGNORE_INSERTS: DWORD = 0x00000200;

pub const LANG_NEUTRAL: USHORT = 0x00;
pub const SUBLANG_DEFAULT: USHORT = 0x01;

#[repr(C)]
pub struct PROCESSENTRY32 {
    pub dwSize: DWORD,
    pub cntUsage: DWORD,
    pub th32ProcessID: DWORD,
    pub th32DefaultHeapID: ULONG_PTR,
    pub th32ModuleID: DWORD,
    pub cntThreads: DWORD,
    pub th32ParentProcessID: DWORD,
    pub pcPriClassBase: LONG,
    pub dwFlags: DWORD,
    pub szExeFile: [CHAR; 260],
}

extern "system" {
    pub fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
    pub fn Process32First(hSnapshot: HANDLE, lppe: LPPROCESSENTRY32) -> BOOL;
    pub fn Process32Next(hSnapshot: HANDLE, lppe: LPPROCESSENTRY32) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: LPDWORD) -> BOOL;
    pub fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn ReadProcessMemory(
        hProcess: HANDLE,
        lpBaseAddress: LPCVOID,
        lpBuffer: LPVOID,
        nSize: SIZE_T,
        lpNumberOfBytesRead: *mut SIZE_T,
    ) -> BOOL;
    pub fn WriteProcessMemory(
        hProcess: HANDLE,
        lpBaseAddress: LPVOID,
        lpBuffer: LPCVOID,
        nSize: SIZE_T,
        lpNumberOfBytesWritten: *mut SIZE_T,
    ) -> BOOL;
    pub fn GetKeyState(nVirtKey: c_int) -> SHORT;
    pub fn VirtualAllocEx(
        hProcess: HANDLE,
        lpAddress: LPVOID,
        dwSize: SIZE_T,
        flAllocationType: DWORD,
        flProtect: DWORD,
    ) -> LPVOID;
    pub fn VirtualProtectEx(
        hProcess: HANDLE,
        lpAddress: LPVOID,
        dwSize: SIZE_T,
        flNewProtect: DWORD,
        lpflOldProtect: PDWORD,
    ) -> BOOL;
    pub fn FormatMessageA(
        dwFlags: DWORD,
        lpSource: LPCVOID,
        dwMessageId: DWORD,
        dwLanguageId: DWORD,
        lpBuffer: LPSTR,
        nSize: DWORD,
        Arguments: *mut va_list,
    ) -> DWORD;
    pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;
}

/// Creates a language identifier from a primary language identifier and a sublanguage identifier.
#[inline]
pub fn MAKELANGID(p: USHORT, s: USHORT) -> LANGID {
    (s << 10) | p
}
