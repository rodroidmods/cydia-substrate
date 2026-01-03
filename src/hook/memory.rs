use crate::error::{Result, SubstrateError};
use libc::{c_void, mmap, mprotect, munmap, sysconf, MAP_ANON, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE, _SC_PAGESIZE};
use std::ptr;

pub struct ProtectedMemory {
    address: *mut u8,
    width: usize,
}

impl ProtectedMemory {
    pub unsafe fn new(data: *mut u8, size: usize) -> Result<Self> {
        if data.is_null() || size == 0 {
            return Err(SubstrateError::NullPointer);
        }

        let page_size = sysconf(_SC_PAGESIZE) as usize;
        let base = (data as usize / page_size) * page_size;
        let width = ((data as usize + size - 1) / page_size + 1) * page_size - base;
        let address = base as *mut u8;

        let result = mprotect(
            address as *mut c_void,
            width,
            PROT_READ | PROT_WRITE | PROT_EXEC,
        );

        if result == -1 {
            let err = std::io::Error::last_os_error();
            return Err(SubstrateError::MemoryProtection(format!("{}", err)));
        }

        Ok(Self { address, width })
    }
}

impl Drop for ProtectedMemory {
    fn drop(&mut self) {
        unsafe {
            mprotect(
                self.address as *mut c_void,
                self.width,
                PROT_READ | PROT_EXEC,
            );

            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            {
                extern "C" {
                    fn __clear_cache(begin: *mut c_void, end: *mut c_void);
                }
                __clear_cache(
                    self.address as *mut c_void,
                    self.address.add(self.width) as *mut c_void,
                );
            }
        }
    }
}

pub unsafe fn allocate_trampoline(size: usize) -> Result<*mut u8> {
    let ptr = mmap(
        ptr::null_mut(),
        size,
        PROT_READ | PROT_WRITE,
        MAP_ANON | MAP_PRIVATE,
        -1,
        0,
    );

    if ptr == MAP_FAILED {
        let err = std::io::Error::last_os_error();
        return Err(SubstrateError::MemoryMap(format!("{}", err)));
    }

    Ok(ptr as *mut u8)
}

pub unsafe fn make_executable(ptr: *mut u8, size: usize) -> Result<()> {
    let result = mprotect(ptr as *mut c_void, size, PROT_READ | PROT_EXEC);

    if result == -1 {
        let err = std::io::Error::last_os_error();
        return Err(SubstrateError::MemoryProtection(format!("{}", err)));
    }

    Ok(())
}

pub unsafe fn free_trampoline(ptr: *mut u8, size: usize) {
    munmap(ptr as *mut c_void, size);
}
