# Quick Start Guide

## For Rust Users

### Installation

```toml
# Add to Cargo.toml
[dependencies]
substrate-rs = "0.1.4"
```

### Basic Usage

```rust
use substrate::hook_function;

unsafe extern "C" fn my_hook(value: i32) -> i32 {
    println!("Hooked! value = {}", value);
    value + 100
}

fn main() {
    unsafe {
        let target = 0x12345678 as *mut u8;
        let hook = my_hook as *mut u8;

        match hook_function(target, hook) {
            Ok(original) => println!("Success! Original: {:p}", original),
            Err(e) => eprintln!("Failed: {}", e),
        }
    }
}
```

## For C/C++ Users

### Download Prebuilt Library

```bash
# Download from GitHub releases
wget https://github.com/YOURUSERNAME/substrate-rs/releases/latest/download/libsubstrate.so

# Or build from source
git clone https://github.com/YOURUSERNAME/substrate-rs
cd substrate-rs
cargo build --release
# Library at: target/release/libsubstrate.so
```

### C Example

```c
#include "substrate.h"

void (*old_func)(void) = NULL;

void my_hook(void) {
    printf("Hooked!\n");
    if (old_func) old_func();
}

int main() {
    uintptr_t addr = getAbsoluteAddress("libgame.so", 0x123456);
    MSHookFunction((void*)addr, (void*)my_hook, (void**)&old_func);
    return 0;
}
```

### Compile

```bash
gcc your_code.c -o your_app -lsubstrate -ldl
```

## Common Patterns

### Android Game Hooking

```rust
use substrate::utils::*;
use substrate::MSHookFunction;

static mut OLD_UPDATE: *mut c_void = std::ptr::null_mut();

unsafe extern "C" fn hooked_update(instance: *mut c_void) {
    // Your code here
    println!("Update called!");

    // Call original
    if !OLD_UPDATE.is_null() {
        let original: extern "C" fn(*mut c_void) =
            std::mem::transmute(OLD_UPDATE);
        original(instance);
    }
}

fn main() {
    unsafe {
        let addr = getAbsoluteAddress("libil2cpp.so", 0x123456)
            .expect("Failed to find address");

        MSHookFunction(
            addr as *mut c_void,
            hooked_update as *mut c_void,
            &mut OLD_UPDATE
        );
    }
}
```

### C++ Style (Just Like Original Substrate)

```cpp
#include "substrate.h"

void (*old_FixedUpdate)(void *instance);

void hooked_FixedUpdate(void *instance) {
    printf("FixedUpdate hooked!\n");
    if (old_FixedUpdate) {
        old_FixedUpdate(instance);
    }
}

int main() {
    // Exactly like your example!
    MSHookFunction(
        (void *)getAbsoluteAddress("libil2cpp.so", 0x123456),
        (void *)hooked_FixedUpdate,
        (void **)&old_FixedUpdate
    );
    return 0;
}
```

## API Quick Reference

### Core Functions

```c
// Hook a function
MSHookFunction(target, hook, &original);

// Find library base
uintptr_t base = findLibrary("libname.so");

// Get absolute address
uintptr_t addr = getAbsoluteAddress("libname.so", 0x1234);

// Check if loaded
if (isLibraryLoaded("libname.so")) { ... }

// Parse hex string
uintptr_t offset = string2Offset("0x123456");
```

### Error Handling (Rust)

```rust
match hook_function(target, hook) {
    Ok(original) => { /* success */ },
    Err(SubstrateError::NullPointer) => { /* null pointer */ },
    Err(SubstrateError::HookFailed(msg)) => { /* hook failed */ },
    Err(e) => { /* other error */ },
}
```

## Architecture Support

- ✅ x86/x86-64 (Intel/AMD)
- ✅ ARMv7 (32-bit ARM + Thumb)
- ✅ ARM64/AArch64 (64-bit ARM)

## Common Scenarios

### Wait for Library

```rust
while !is_library_loaded("libtarget.so") {
    std::thread::sleep(std::time::Duration::from_millis(100));
}
```

### Multiple Hooks

```rust
unsafe {
    MSHookFunction(addr1, hook1, &mut orig1);
    MSHookFunction(addr2, hook2, &mut orig2);
    MSHookFunction(addr3, hook3, &mut orig3);
}
```

### Debug Mode

```rust
use substrate::set_debug;
set_debug(true);  // Enable debug logging
```

## Troubleshooting

### Hook Not Working?

1. Verify address is correct: `printf("0x%lx\n", addr);`
2. Check if library is loaded: `isLibraryLoaded("lib.so")`
3. Enable debug mode: `set_debug(true)`
4. Verify architecture matches

### Library Not Found?

```bash
# Check loaded libraries
cat /proc/self/maps | grep libname

# On Android
adb shell cat /proc/PID/maps | grep libname
```

### Segmentation Fault?

- Make sure addresses are valid
- Check function signatures match
- Verify architecture (ARM vs ARM64)
- Enable debug logging

## Next Steps

- Read full [README.md](README.md) for detailed documentation
- Check [examples/](examples/) for more code samples
- See [PUBLISHING.md](PUBLISHING.md) for advanced topics
- Visit [docs.rs](https://docs.rs/substrate-rs) for API documentation

## Support

- 📚 [Documentation](https://docs.rs/substrate-rs)
- 🐛 [Issues](https://github.com/rodroidmods/cydia-substrate/issues)
- 💬 [Discussions](https://github.com/rodroidmods/cydia-substrate/discussions)

## License

LGPL-3.0 - Same as original Cydia Substrate
