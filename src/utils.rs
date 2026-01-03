use crate::error::{Result, SubstrateError};
use std::ffi::CStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::raw::{c_char, c_void};

/// Find the base address of a loaded library in the current process.
///
/// Searches through `/proc/self/maps` to locate the library and returns its base address.
///
/// # Arguments
///
/// * `library_name` - Name of the library to find (e.g., "libil2cpp.so")
///
/// # Returns
///
/// `Ok(usize)` containing the base address of the library.
/// `Err(SubstrateError::LibraryNotFound)` if the library is not loaded.
///
/// # Examples
///
/// ```no_run
/// use substrate::utils::find_library;
///
/// let base = find_library("libil2cpp.so").expect("Library not found");
/// println!("libil2cpp.so base: 0x{:x}", base);
/// ```
pub fn find_library(library_name: &str) -> Result<usize> {
    let maps_file = File::open("/proc/self/maps").map_err(|e| {
        SubstrateError::FileNotFound(format!("Failed to open /proc/self/maps: {}", e))
    })?;

    let reader = BufReader::new(maps_file);

    for line in reader.lines() {
        let line = line.map_err(|e| {
            SubstrateError::ParseError(format!("Failed to read line: {}", e))
        })?;

        if line.contains(library_name) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                if let Some(addr_str) = parts[0].split('-').next() {
                    return usize::from_str_radix(addr_str, 16).map_err(|e| {
                        SubstrateError::ParseError(format!("Failed to parse address: {}", e))
                    });
                }
            }
        }
    }

    Err(SubstrateError::LibraryNotFound(library_name.to_string()))
}

/// Convert a relative offset to an absolute address by adding the library base address.
///
/// This is the most commonly used function for hooking, combining library lookup and offset calculation.
///
/// # Arguments
///
/// * `library_name` - Name of the library (e.g., "libil2cpp.so")
/// * `relative_addr` - Offset from the library base (e.g., 0x123456)
///
/// # Returns
///
/// `Ok(usize)` containing the absolute address.
/// `Err(SubstrateError)` if the library cannot be found.
///
/// # Examples
///
/// ```no_run
/// use substrate::utils::get_absolute_address;
///
/// let addr = get_absolute_address("libil2cpp.so", 0x123456)
///     .expect("Failed to get address");
/// println!("Absolute address: 0x{:x}", addr);
/// ```
pub fn get_absolute_address(library_name: &str, relative_addr: usize) -> Result<usize> {
    let base = find_library(library_name)?;
    Ok(base + relative_addr)
}

/// Check if a library is currently loaded in the process.
///
/// # Arguments
///
/// * `library_name` - Name of the library to check
///
/// # Returns
///
/// `true` if the library is loaded, `false` otherwise.
///
/// # Examples
///
/// ```no_run
/// use substrate::utils::is_library_loaded;
///
/// if is_library_loaded("libil2cpp.so") {
///     println!("IL2CPP is loaded!");
/// }
/// ```
pub fn is_library_loaded(library_name: &str) -> bool {
    if let Ok(file) = File::open("/proc/self/maps") {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if line.contains(library_name) {
                return true;
            }
        }
    }
    false
}

/// Parse a hexadecimal string to a numeric offset.
///
/// Accepts strings with or without "0x" prefix.
///
/// # Arguments
///
/// * `s` - String containing hexadecimal number (e.g., "0x123456" or "123456")
///
/// # Returns
///
/// `Ok(usize)` containing the parsed offset.
/// `Err(SubstrateError::ParseError)` if parsing fails.
///
/// # Examples
///
/// ```no_run
/// use substrate::utils::string_to_offset;
///
/// let offset = string_to_offset("0x123456").expect("Parse failed");
/// assert_eq!(offset, 0x123456);
/// ```
pub fn string_to_offset(s: &str) -> Result<usize> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(s, 16).map_err(|e| {
        SubstrateError::ParseError(format!("Failed to parse offset: {}", e))
    })
}

/// C FFI: Find the base address of a loaded library.
///
/// # Safety
///
/// The `library` parameter must be a valid null-terminated C string.
///
/// # Returns
///
/// Base address of the library, or 0 if not found.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn findLibrary(library: *const c_char) -> usize { unsafe {
    if library.is_null() {
        return 0;
    }

    let library_name = match CStr::from_ptr(library).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    find_library(library_name).unwrap_or(0)
}}

/// C FFI: Convert a relative offset to an absolute address.
///
/// This is the most commonly used function from C/C++ for hooking.
///
/// # Safety
///
/// The `library` parameter must be a valid null-terminated C string.
///
/// # Returns
///
/// Absolute address, or 0 if the library cannot be found.
///
/// # Examples (C)
///
/// ```c
/// uintptr_t addr = getAbsoluteAddress("libil2cpp.so", 0x123456);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getAbsoluteAddress(library: *const c_char, offset: usize) -> usize { unsafe {
    if library.is_null() {
        return 0;
    }

    let library_name = match CStr::from_ptr(library).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    get_absolute_address(library_name, offset).unwrap_or(0)
}}

/// C FFI: Check if a library is loaded in the current process.
///
/// # Safety
///
/// The `library` parameter must be a valid null-terminated C string.
///
/// # Returns
///
/// `true` if loaded, `false` otherwise.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn isLibraryLoaded(library: *const c_char) -> bool { unsafe {
    if library.is_null() {
        return false;
    }

    let library_name = match CStr::from_ptr(library).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    is_library_loaded(library_name)
}}

/// C FFI: Parse a hexadecimal string to a numeric offset.
///
/// # Safety
///
/// The `s` parameter must be a valid null-terminated C string.
///
/// # Returns
///
/// Parsed offset value, or 0 if parsing fails.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn string2Offset(s: *const c_char) -> usize { unsafe {
    if s.is_null() {
        return 0;
    }

    let string = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    string_to_offset(string).unwrap_or(0)
}}

/// C FFI: Convenience function for hooking with automatic architecture detection.
///
/// This function automatically selects the appropriate hooking implementation
/// based on the target architecture.
///
/// # Safety
///
/// Same safety requirements as `MSHookFunction`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hook(
    offset: *mut c_void,
    ptr: *mut c_void,
    orig: *mut *mut c_void,
) { unsafe {
    #[cfg(target_arch = "aarch64")]
    {
        crate::A64HookFunction(offset, ptr, orig);
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        crate::MSHookFunction(offset, ptr, orig);
    }
}}
