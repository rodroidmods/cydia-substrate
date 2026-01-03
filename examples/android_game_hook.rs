use substrate::utils::{get_absolute_address, is_library_loaded, string_to_offset};
use substrate::MSHookFunction;
use std::ffi::c_void;
use std::thread;
use std::time::Duration;

static mut OLD_UPDATE: *mut c_void = std::ptr::null_mut();
static mut OLD_FIXED_UPDATE: *mut c_void = std::ptr::null_mut();

#[repr(C)]
struct UnityObject {
    vtable: *mut c_void,
    data: *mut c_void,
}

unsafe extern "C" fn hooked_update(instance: *mut UnityObject) {
    println!("[Hook] Update() called - instance: {:p}", instance);

    if !OLD_UPDATE.is_null() {
        let original: extern "C" fn(*mut UnityObject) = std::mem::transmute(OLD_UPDATE);
        original(instance);
    }
}

unsafe extern "C" fn hooked_fixed_update(instance: *mut UnityObject) {
    println!("[Hook] FixedUpdate() called - instance: {:p}", instance);

    if !OLD_FIXED_UPDATE.is_null() {
        let original: extern "C" fn(*mut UnityObject) = std::mem::transmute(OLD_FIXED_UPDATE);
        original(instance);
    }
}

fn wait_for_library(lib_name: &str, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if is_library_loaded(lib_name) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

fn main() {
    println!("=== Android Game Hook Example (IL2CPP/Unity) ===\n");

    let library = "libil2cpp.so";

    let update_offset_str = "0x123456";
    let fixed_update_offset_str = "0x789ABC";

    println!("Waiting for {} to load...", library);

    if wait_for_library(library, 30) {
        println!("✓ {} loaded!", library);

        unsafe {
            match string_to_offset(update_offset_str) {
                Ok(update_offset) => {
                    println!("\nHooking Update() at offset: 0x{:X}", update_offset);

                    match get_absolute_address(library, update_offset) {
                        Ok(addr) => {
                            println!("Absolute address: 0x{:x}", addr);

                            MSHookFunction(
                                addr as *mut c_void,
                                hooked_update as *mut c_void,
                                &mut OLD_UPDATE
                            );

                            if !OLD_UPDATE.is_null() {
                                println!("✓ Update() hooked successfully!");
                            }
                        }
                        Err(e) => eprintln!("✗ Failed to get address: {}", e),
                    }
                }
                Err(e) => eprintln!("✗ Invalid offset: {}", e),
            }

            match string_to_offset(fixed_update_offset_str) {
                Ok(fixed_offset) => {
                    println!("\nHooking FixedUpdate() at offset: 0x{:X}", fixed_offset);

                    match get_absolute_address(library, fixed_offset) {
                        Ok(addr) => {
                            println!("Absolute address: 0x{:x}", addr);

                            MSHookFunction(
                                addr as *mut c_void,
                                hooked_fixed_update as *mut c_void,
                                &mut OLD_FIXED_UPDATE
                            );

                            if !OLD_FIXED_UPDATE.is_null() {
                                println!("✓ FixedUpdate() hooked successfully!");
                            }
                        }
                        Err(e) => eprintln!("✗ Failed to get address: {}", e),
                    }
                }
                Err(e) => eprintln!("✗ Invalid offset: {}", e),
            }

            println!("\n=== Hooks Installed ===");
            println!("The game functions will now call your hooks.");
            println!("\nNote: Replace the offset values with real ones from your game!");
        }
    } else {
        eprintln!("✗ Timeout waiting for {} to load", library);
        eprintln!("\nTo use this example:");
        eprintln!("1. Find function offsets using IDA Pro, Ghidra, or similar");
        eprintln!("2. Replace the offset strings with actual values");
        eprintln!("3. Run as part of an injected library in the game process");
    }
}
