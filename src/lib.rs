#![doc = include_str!("../README.md")]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod arch;
pub mod debug;
pub mod disasm;
pub mod error;
pub mod hook;
pub mod symbol;
pub mod utils;

use error::{Result, SubstrateError};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

pub type MSImageRef = *const c_void;

static MS_DEBUG: AtomicBool = AtomicBool::new(false);

#[unsafe(no_mangle)]
pub static mut MSDebug: bool = false;

pub fn set_debug(enabled: bool) {
    MS_DEBUG.store(enabled, Ordering::Relaxed);
    unsafe { MSDebug = enabled; }
}

pub fn is_debug() -> bool {
    MS_DEBUG.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSHookFunction(
    symbol: *mut c_void,
    replace: *mut c_void,
    result: *mut *mut c_void,
) { unsafe {
    if symbol.is_null() {
        return;
    }

    let result_ptr = if result.is_null() {
        ptr::null_mut()
    } else {
        result as *mut *mut u8
    };

    #[cfg(target_arch = "x86_64")]
    {
        let _ = arch::x86_64::hook_function_x86_64(
            symbol as *mut u8,
            replace as *mut u8,
            result_ptr,
        );
    }

    #[cfg(target_arch = "arm")]
    {
        let symbol_addr = symbol as usize;
        if (symbol_addr & 0x1) == 0 {
            let _ = arch::arm::hook_function_arm(
                symbol as *mut u8,
                replace as *mut u8,
                result_ptr,
            );
        } else {
            let _ = arch::thumb::hook_function_thumb(
                (symbol_addr & !0x1) as *mut u8,
                replace as *mut u8,
                result_ptr,
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        let _ = arch::aarch64::hook_function_aarch64(
            symbol as *mut u8,
            replace as *mut u8,
            result_ptr,
        );
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn A64HookFunction(
    symbol: *mut c_void,
    replace: *mut c_void,
    result: *mut *mut c_void,
) { unsafe {
    MSHookFunction(symbol, replace, result);
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSFindSymbol(image: MSImageRef, name: *const c_char) -> *mut c_void { unsafe {
    if name.is_null() {
        return ptr::null_mut();
    }

    let symbol_name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    if image.is_null() {
        return ptr::null_mut();
    }

    let library_path = CStr::from_ptr(image as *const c_char).to_str().unwrap_or("");
    if library_path.is_empty() {
        return ptr::null_mut();
    }

    let pid = std::process::id() as i32;
    match symbol::finder::find_symbol_address(pid, symbol_name, library_path) {
        Ok(addr) => addr as *mut c_void,
        Err(_) => ptr::null_mut(),
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSGetImageByName(file: *const c_char) -> MSImageRef { unsafe {
    if file.is_null() {
        return ptr::null();
    }

    let library_name = match CStr::from_ptr(file).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    let pid = std::process::id() as i32;
    match symbol::memmap::find_library_base(pid, library_name) {
        Ok(_) => file as MSImageRef,
        Err(_) => ptr::null(),
    }
}}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSHookProcess(_pid: c_int, _library: *const c_char) -> bool {
    false
}

pub unsafe fn hook_function<T>(symbol: *mut T, replace: *mut T) -> Result<*mut T> { unsafe {
    if symbol.is_null() || replace.is_null() {
        return Err(SubstrateError::NullPointer);
    }

    let mut result: *mut T = ptr::null_mut();

    #[cfg(target_arch = "x86_64")]
    {
        arch::x86_64::hook_function_x86_64(
            symbol as *mut u8,
            replace as *mut u8,
            &mut result as *mut *mut T as *mut *mut u8,
        )?;
    }

    #[cfg(target_arch = "arm")]
    {
        let symbol_addr = symbol as usize;
        if (symbol_addr & 0x1) == 0 {
            arch::arm::hook_function_arm(
                symbol as *mut u8,
                replace as *mut u8,
                &mut result as *mut *mut T as *mut *mut u8,
            )?;
        } else {
            arch::thumb::hook_function_thumb(
                (symbol_addr & !0x1) as *mut u8,
                replace as *mut u8,
                &mut result as *mut *mut T as *mut *mut u8,
            )?;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        arch::aarch64::hook_function_aarch64(
            symbol as *mut u8,
            replace as *mut u8,
            &mut result as *mut *mut T as *mut *mut u8,
        )?;
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64")))]
    {
        return Err(SubstrateError::HookFailed("Architecture not implemented".to_string()));
    }

    Ok(result)
}}

pub fn find_symbol_in_process(
    pid: libc::pid_t,
    library: &str,
    symbol: &str,
) -> Result<*mut c_void> {
    let addr = symbol::finder::find_symbol_address(pid, symbol, library)?;
    Ok(addr as *mut c_void)
}
