use crate::proc;
use crate::winapi;

const JMP_LEN: usize = 5;

pub fn godmode() -> InjectionSpec {
    InjectionSpec {
        original_addr: 0x429d1f,
        original_code: vec![
            // 00000000 <originalcode>:
            0x29, 0x7b, 0x04, // sub [ebx+0x4],edi
            0x8b, 0xc7, // mov eax, edi
        ],
        new_code: vec![
            // 00000000 <newcode>:
            0xa1, 0xf4, 0xf4, 0x50, 0x00, // mov eax, [0x50f4f4]
            0x05, 0xf4, 0x00, 0x00, 0x00, // add eax, 0xf4
            0x39, 0xc3, // cmp ebx, eax
            0x74, 0x03, // je 0x11 <originalcode+0x3>
            // 0000000e <originalcode>:
            0x29, 0x7b, 0x04, // sub [ebx+0x4],edi
            // 00000011 <originalcode+0x3>:
            0x8b, 0xc7, // mov eax, edi
        ],
    }
}

pub struct InjectionSpec {
    original_addr: u32,
    original_code: Vec<u8>,
    new_code: Vec<u8>,
}

impl InjectionSpec {
    pub fn inject(self, handle: proc::Handle) -> Result<Injection, String> {
        // TODO: check if original code in the process matches self.original_code before
        // continuing

        let new_code_addr = unsafe {
            winapi::VirtualAllocEx(
                handle,
                std::ptr::null_mut(),
                self.new_code.len(),
                winapi::MEM_COMMIT | winapi::MEM_RESERVE,
                winapi::PAGE_EXECUTE_READWRITE,
            ) as u32
        };

        let mut new_code = self.new_code;
        new_code.extend_from_slice(
            &jmp(
                new_code_addr + new_code.len() as u32,
                self.original_addr + JMP_LEN as u32,
            )[..],
        );

        proc::write(handle, new_code_addr, &new_code)?;

        Ok(Injection {
            original_addr: self.original_addr,
            original_code: self.original_code,
            new_code_addr,
        })
    }
}

pub struct Injection {
    original_addr: u32,
    original_code: Vec<u8>,
    new_code_addr: u32,
}

impl Injection {
    pub fn enable(&self, handle: proc::Handle) -> Result<(), String> {
        let jmp_partial = jmp(self.original_addr, self.new_code_addr);
        let mut jmp_code = vec![0x90; self.original_code.len()];
        jmp_code[..JMP_LEN].clone_from_slice(&jmp_partial[..]);
        write_code(handle, self.original_addr, &jmp_code[..])
    }

    pub fn disable(&self, handle: proc::Handle) -> Result<(), String> {
        write_code(handle, self.original_addr, &self.original_code)
    }
}

fn write_code(handle: proc::Handle, addr: u32, code: &[u8]) -> Result<(), String> {
    // TODO: check for VirtualProtectEx errors

    let mut old_protection: winapi::DWORD = 0;

    unsafe {
        winapi::VirtualProtectEx(
            handle,
            addr as winapi::LPVOID,
            code.len(),
            winapi::PAGE_EXECUTE_READWRITE,
            &mut old_protection as *mut _ as winapi::PDWORD,
        );
    }

    proc::write(handle, addr, &code)?;

    unsafe {
        winapi::VirtualProtectEx(
            handle,
            addr as winapi::LPVOID,
            code.len(),
            old_protection,
            std::ptr::null_mut(),
        );
    }

    Ok(())
}

fn jmp(src: u32, dst: u32) -> [u8; JMP_LEN] {
    let offset = dst as i32 - (src as i32 + JMP_LEN as i32);
    let offset: [u8; 4] = unsafe { std::mem::transmute(offset) };
    [0xe9, offset[0], offset[1], offset[2], offset[3]]
}
