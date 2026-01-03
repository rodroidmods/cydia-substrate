use substrate::{hook_function, utils};
use substrate::error::SubstrateError;

fn demonstrate_error_handling() {
    println!("=== Error Handling Example ===\n");

    unsafe {
        println!("1. Testing null pointer hook:");
        match hook_function(std::ptr::null_mut::<u8>(), std::ptr::null_mut()) {
            Ok(_) => println!("  Unexpected success"),
            Err(SubstrateError::NullPointer) => println!("  ✓ Correctly caught null pointer"),
            Err(e) => println!("  Different error: {}", e),
        }

        println!("\n2. Testing library not found:");
        match utils::find_library("nonexistent.so") {
            Ok(_) => println!("  Unexpected success"),
            Err(SubstrateError::LibraryNotFound(name)) => {
                println!("  ✓ Correctly caught missing library: {}", name)
            }
            Err(e) => println!("  Different error: {}", e),
        }

        println!("\n3. Testing invalid offset string:");
        match utils::string_to_offset("not_a_hex") {
            Ok(_) => println!("  Unexpected success"),
            Err(SubstrateError::ParseError(msg)) => {
                println!("  ✓ Correctly caught parse error: {}", msg)
            }
            Err(e) => println!("  Different error: {}", e),
        }

        println!("\n4. Testing valid hex parsing:");
        match utils::string_to_offset("0x123ABC") {
            Ok(offset) => println!("  ✓ Successfully parsed: 0x{:X}", offset),
            Err(e) => println!("  ✗ Unexpected error: {}", e),
        }
    }

    println!("\n=== All Error Cases Handled ===");
}

fn main() {
    demonstrate_error_handling();

    println!("\n=== Comprehensive Error Handling ===\n");
    println!("Always handle errors properly in production code:");
    println!();
    println!("match hook_function(target, hook) {{");
    println!("    Ok(original) => {{");
    println!("        // Success - use original");
    println!("    }}");
    println!("    Err(SubstrateError::NullPointer) => {{");
    println!("        // Handle null pointer");
    println!("    }}");
    println!("    Err(SubstrateError::HookFailed(msg)) => {{");
    println!("        // Handle hook failure");
    println!("    }}");
    println!("    Err(e) => {{");
    println!("        // Handle other errors");
    println!("    }}");
    println!("}}");
}
