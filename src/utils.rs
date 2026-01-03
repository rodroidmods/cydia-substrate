use crate::error::{Result, SubstrateError};
use std::ffi::CStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::raw::{c_char, c_void};

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

pub fn get_absolute_address(library_name: &str, relative_addr: usize) -> Result<usize> {
    let base = find_library(library_name)?;
    Ok(base + relative_addr)
}

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

pub fn string_to_offset(s: &str) -> Result<usize> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(s, 16).map_err(|e| {
        SubstrateError::ParseError(format!("Failed to parse offset: {}", e))
    })
}

#[no_mangle]
pub unsafe extern "C" fn findLibrary(library: *const c_char) -> usize {
    if library.is_null() {
        return 0;
    }

    let library_name = match CStr::from_ptr(library).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    find_library(library_name).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn getAbsoluteAddress(library: *const c_char, offset: usize) -> usize {
    if library.is_null() {
        return 0;
    }

    let library_name = match CStr::from_ptr(library).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    get_absolute_address(library_name, offset).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn isLibraryLoaded(library: *const c_char) -> bool {
    if library.is_null() {
        return false;
    }

    let library_name = match CStr::from_ptr(library).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    is_library_loaded(library_name)
}

#[no_mangle]
pub unsafe extern "C" fn string2Offset(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }

    let string = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    string_to_offset(string).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn hook(
    offset: *mut c_void,
    ptr: *mut c_void,
    orig: *mut *mut c_void,
) {
    #[cfg(target_arch = "aarch64")]
    {
        crate::A64HookFunction(offset, ptr, orig);
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        crate::MSHookFunction(offset, ptr, orig);
    }
}
