use substrate;
use std::ffi::c_void;

static mut ORIGINAL: *mut c_void = std::ptr::null_mut();

unsafe extern "C" fn my_hook() {
    println!("Hook executed!");
}

fn main() {
    println!("=== Multi-Architecture Hook Example ===\n");

    #[cfg(target_arch = "x86_64")]
    println!("Architecture: x86-64 (64-bit Intel/AMD)");

    #[cfg(target_arch = "x86")]
    println!("Architecture: x86 (32-bit Intel/AMD)");

    #[cfg(target_arch = "arm")]
    println!("Architecture: ARMv7 (32-bit ARM with Thumb)");

    #[cfg(target_arch = "aarch64")]
    println!("Architecture: ARM64/AArch64 (64-bit ARM)");

    println!("\nThis example shows architecture-specific behavior.\n");

    unsafe {
        let target: *mut c_void = std::ptr::null_mut();

        #[cfg(target_arch = "aarch64")]
        {
            println!("Using A64HookFunction for ARM64...");
            substrate::A64HookFunction(
                target,
                my_hook as *mut c_void,
                &mut ORIGINAL
            );
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            println!("Using MSHookFunction...");
            substrate::MSHookFunction(
                target,
                my_hook as *mut c_void,
                &mut ORIGINAL
            );
        }

        println!("Hook API called successfully!");
        println!("\nNote: This example uses null pointers for demonstration.");
        println!("In real usage, provide valid function addresses.");
    }
}
