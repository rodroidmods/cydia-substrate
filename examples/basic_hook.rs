use substrate::hook_function;
use std::sync::atomic::{AtomicUsize, Ordering};

static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
static mut ORIGINAL_ADD: *mut u8 = std::ptr::null_mut();

unsafe extern "C" fn original_add(a: i32, b: i32) -> i32 {
    a + b
}

unsafe extern "C" fn hooked_add(a: i32, b: i32) -> i32 {
    CALL_COUNT.fetch_add(1, Ordering::Relaxed);
    println!("hooked_add called with: {} + {}", a, b);

    if !ORIGINAL_ADD.is_null() {
        let original: extern "C" fn(i32, i32) -> i32 = std::mem::transmute(ORIGINAL_ADD);
        let result = original(a, b);
        println!("Original result: {}", result);
        result + 1000
    } else {
        a + b + 1000
    }
}

fn main() {
    println!("=== Basic Function Hook Example ===\n");

    unsafe {
        let target = original_add as *mut u8;
        let hook = hooked_add as *mut u8;

        println!("Target function: {:p}", target);
        println!("Hook function: {:p}", hook);

        let result = original_add(5, 3);
        println!("Before hook: 5 + 3 = {}", result);

        match hook_function(target, hook) {
            Ok(original) => {
                ORIGINAL_ADD = original;
                println!("\n✓ Hook installed successfully!");
                println!("Trampoline (original): {:p}\n", original);

                let result = original_add(5, 3);
                println!("After hook: 5 + 3 = {}", result);

                let calls = CALL_COUNT.load(Ordering::Relaxed);
                println!("\nTotal hooked calls: {}", calls);
            }
            Err(e) => {
                eprintln!("✗ Hook failed: {}", e);
            }
        }
    }
}
