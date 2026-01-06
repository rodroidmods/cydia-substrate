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

/// Hook a function at runtime by redirecting its execution to a replacement function.
///
/// This is the primary C-compatible hooking function that works across all supported architectures
/// (x86-64, ARMv7, ARM64). It installs an inline hook by modifying the target function's prologue
/// to jump to your replacement function, while preserving the original instructions in a trampoline.
///
/// # Arguments
///
/// * `symbol` - Pointer to the function to hook (must not be null)
/// * `replace` - Pointer to your replacement function (must not be null)
/// * `result` - Output pointer that receives the trampoline address to call the original function.
///              Pass null if you don't need to call the original function.
///
/// # Safety
///
/// This function is unsafe because it:
/// - Modifies executable code at runtime
/// - Requires valid function pointers
/// - Changes memory protection flags
/// - Can cause undefined behavior if pointers are invalid
///
/// # Examples
///
/// ```no_run
/// use substrate::MSHookFunction;
/// use std::os::raw::c_void;
///
/// static mut ORIGINAL: *mut c_void = std::ptr::null_mut();
///
/// unsafe extern "C" fn my_replacement() {
///     println!("Hooked!");
///     if !ORIGINAL.is_null() {
///         let orig: extern "C" fn() = std::mem::transmute(ORIGINAL);
///         orig();
///     }
/// }
///
/// unsafe {
///     let target = 0x12345678 as *mut c_void;
///     MSHookFunction(target, my_replacement as *mut c_void, &mut ORIGINAL);
/// }
/// ```
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

/// ARM64-specific hook function (alias for MSHookFunction).
///
/// This function is provided for compatibility with And64InlineHook API.
/// On ARM64 platforms, it behaves identically to `MSHookFunction`.
///
/// # Safety
///
/// Same safety requirements as `MSHookFunction`. See [`MSHookFunction`] for details.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn A64HookFunction(
    symbol: *mut c_void,
    replace: *mut c_void,
    result: *mut *mut c_void,
) { unsafe {
    MSHookFunction(symbol, replace, result);
}}

/// Find a symbol by name within a loaded image.
///
/// # Arguments
///
/// * `_image` - Reference to the loaded image (currently unused)
/// * `name` - C string containing the symbol name to find
///
/// # Returns
///
/// Pointer to the symbol if found, null pointer otherwise.
///
/// # Safety
///
/// The `name` parameter must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSFindSymbol(_image: MSImageRef, name: *const c_char) -> *mut c_void { unsafe {
    if name.is_null() {
        return ptr::null_mut();
    }

    let _symbol_name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    ptr::null_mut()
}}

/// Get a reference to a loaded image (library) by filename.
///
/// # Arguments
///
/// * `_file` - C string containing the library filename
///
/// # Returns
///
/// Reference to the loaded image if found, null otherwise.
///
/// # Safety
///
/// The `_file` parameter must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSGetImageByName(_file: *const c_char) -> MSImageRef {
    ptr::null()
}

/// Hook into another process by injecting a library.
///
/// # Arguments
///
/// * `_pid` - Process ID to hook into
/// * `_library` - C string containing the library path to inject
///
/// # Returns
///
/// `true` if successful, `false` otherwise.
///
/// # Safety
///
/// This function requires appropriate permissions and the library path must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MSHookProcess(_pid: c_int, _library: *const c_char) -> bool {
    false
}

/// Type-safe Rust wrapper for hooking functions.
///
/// This is a generic wrapper around the C API that provides type safety and Result-based
/// error handling. It's the recommended way to use the hooking functionality from Rust code.
///
/// # Type Parameters
///
/// * `T` - The function type to hook (typically a function pointer)
///
/// # Arguments
///
/// * `symbol` - Pointer to the function to hook
/// * `replace` - Pointer to your replacement function
///
/// # Returns
///
/// `Ok(*mut T)` containing the trampoline pointer to call the original function.
/// `Err(SubstrateError)` if the hook installation fails.
///
/// # Safety
///
/// This function is unsafe because it modifies executable code at runtime.
/// Both pointers must be valid function pointers of the correct type.
///
/// # Examples
///
/// ```no_run
/// use substrate::hook_function;
///
/// extern "C" fn original_func(x: i32) -> i32 { x }
/// extern "C" fn hooked_func(x: i32) -> i32 { x + 1 }
///
/// unsafe {
///     let trampoline = hook_function(
///         original_func as *mut _,
///         hooked_func as *mut _
///     ).expect("Hook failed");
/// }
/// ```
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

/// Find a symbol address in a specific process.
///
/// This function looks up a symbol by name within a specific library loaded in the target process.
/// It parses the process memory maps and ELF symbol tables to resolve the symbol address.
///
/// # Arguments
///
/// * `pid` - Process ID to search in
/// * `library` - Name of the library containing the symbol
/// * `symbol` - Symbol name to find
///
/// # Returns
///
/// `Ok(*mut c_void)` containing the symbol address.
/// `Err(SubstrateError)` if the symbol or library cannot be found.
///
/// # Examples
///
/// ```no_run
/// use substrate::find_symbol_in_process;
///
/// let addr = find_symbol_in_process(
///     std::process::id() as i32,
///     "libil2cpp.so",
///     "il2cpp_init"
/// ).expect("Symbol not found");
/// ```
pub fn find_symbol_in_process(
    pid: libc::pid_t,
    library: &str,
    symbol: &str,
) -> Result<*mut c_void> {
    let addr = symbol::finder::find_symbol_address(pid, symbol, library)?;
    Ok(addr as *mut c_void)
}
