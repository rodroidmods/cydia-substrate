use substrate::utils;
use substrate::MSHookFunction;
use std::ffi::c_void;

static mut OLD_FUNCTION: *mut c_void = std::ptr::null_mut();

unsafe extern "C" fn my_hooked_function(value: i32) -> i32 {
    println!("✓ Hook executed! Input: {}", value);

    if !OLD_FUNCTION.is_null() {
        let original: extern "C" fn(i32) -> i32 = std::mem::transmute(OLD_FUNCTION);
        original(value)
    } else {
        value * 2
    }
}

fn main() {
    println!("=== Library Function Hook Example ===\n");

    println!("This example demonstrates hooking a function in a shared library.");
    println!("Note: This requires a target library to be loaded.\n");

    let library_name = "libexample.so";
    let function_offset = 0x1234;

    unsafe {
        println!("Checking if {} is loaded...", library_name);

        if utils::is_library_loaded(library_name) {
            println!("✓ Library is loaded!");

            match utils::get_absolute_address(library_name, function_offset) {
                Ok(addr) => {
                    println!("Target address: 0x{:x}", addr);
                    println!("Installing hook...");

                    MSHookFunction(
                        addr as *mut c_void,
                        my_hooked_function as *mut c_void,
                        &mut OLD_FUNCTION
                    );

                    println!("✓ Hook installed!");
                    println!("Original function: {:p}", OLD_FUNCTION);
                }
                Err(e) => {
                    eprintln!("✗ Failed to get address: {}", e);
                }
            }
        } else {
            println!("✗ Library '{}' is not loaded", library_name);
            println!("\nTo use this example:");
            println!("1. Replace 'libexample.so' with your target library");
            println!("2. Replace 0x1234 with the actual function offset");
            println!("3. Make sure the library is loaded before running");
        }
    }
}
