# Substrate - Powerful Code Injection Platform

# Note

Version 0.1.7 is the official stable release; earlier versions are not stable.

[![Crates.io](https://img.shields.io/crates/v/substrate-rs.svg)](https://crates.io/crates/substrate-rs)
[![Documentation](https://docs.rs/substrate-rs/badge.svg)](https://docs.rs/substrate-rs)
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)

A complete Rust rewrite of Cydia Substrate and And64InlineHook, providing powerful function hooking capabilities for Android and Linux platforms across multiple architectures.

## Features

- ✅ **Multi-Architecture Support**
  - x86/x86-64 (Intel/AMD 32-bit and 64-bit)
  - ARMv7 (32-bit ARM with Thumb/Thumb-2)
  - ARM64/AArch64 (64-bit ARM)

- ✅ **Complete Function Hooking**
  - Inline function hooking with automatic trampoline generation
  - PC-relative instruction relocation
  - Symbol resolution from ELF binaries
  - Library base address lookup

- ✅ **Dual API**
  - C-compatible FFI for use in C/C++ projects
  - Safe, idiomatic Rust API
  - Drop-in replacement for Cydia Substrate

- ✅ **Production Ready**
  - Zero unsafe behavior leaks
  - Comprehensive error handling
  - Memory-safe by default
  - Thoroughly tested across architectures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
substrate-rs = "0.1.7"
```

Or for C/C++ projects, download the prebuilt library from releases.

## Quick Start

### Rust Usage

```rust
use core::ffi::c_void;
use substrate::{MSHookFunction, utils::getAbsoluteAddress};

static mut LEVEL_BONUS: bool = false;
static mut ORIG_RACE_UPDATE: *mut c_void = core::ptr::null_mut();

unsafe extern "C" fn hook_RaceUpdate(raceManager: *mut c_void, deltaTime: f32) {
    if LEVEL_BONUS && !raceManager.is_null() {
        *((raceManager as *mut u8).add(0x4EC) as *mut f32) = 999.0;
    }

    if !ORIG_RACE_UPDATE.is_null() {
        let orig: extern "C" fn(*mut c_void, f32) = core::mem::transmute(ORIG_RACE_UPDATE);
        orig(raceManager, deltaTime);
    }
}

fn main() {
    unsafe {
        let target = getAbsoluteAddress(b"libil2cpp.so\0".as_ptr() as *const i8, 0x19ADD4);
        MSHookFunction(
            target as *mut c_void,
            hook_RaceUpdate as *mut c_void,
            core::ptr::addr_of_mut!(ORIG_RACE_UPDATE),
        );
    }
}
```

### C/C++ Usage

```cpp
#include <substrate.h>
#include <stdint.h>
#include <stdbool.h>

bool levelbonus = false;

void (*orig_RaceUpdate)(void* raceManager, float deltaTime);

void hook_RaceUpdate(void* raceManager, float deltaTime) {
    if (levelbonus && raceManager != NULL) {
        *(float*)((uintptr_t)raceManager + 0x4EC) = 999.0f;
    }
    orig_RaceUpdate(raceManager, deltaTime);
}

void (*old_update)(void *instance);
void* (*GetCurrentCH)(void*);

void update(void *instance) {
    if (instance != NULL) {
        bool isBlabla = true;
        if (isBlabla) {
            void* getClass = GetCurrentCH(instance);
            if (getClass != NULL) {
                *(bool*)((uintptr_t)getClass + 0xB4) = true;
            }
        }
    }
    old_update(instance);
}

__attribute__((constructor))
static void init_hooks() {
    GetCurrentCH = (void* (*)(void*))getAbsoluteAddress("libil2cpp.so", 0xC10738);

    MSHookFunction(
        (void*)getAbsoluteAddress("libil2cpp.so", 0x19ADD4),
        (void*)hook_RaceUpdate,
        (void**)&orig_RaceUpdate
    );

    MSHookFunction(
        (void*)getAbsoluteAddress("libil2cpp.so", 0xC1123C),
        (void*)update,
        (void**)&old_update
    );
}
```

## Complete Examples

### Example 1: Hooking with Symbol Name

```rust
use core::ffi::c_void;
use substrate::{MSHookFunction, utils::getAbsoluteAddress};

static mut ORIG_MALLOC: *mut c_void = core::ptr::null_mut();

unsafe extern "C" fn hook_malloc(size: usize) -> *mut c_void {
    if !ORIG_MALLOC.is_null() {
        let orig: extern "C" fn(usize) -> *mut c_void = core::mem::transmute(ORIG_MALLOC);
        return orig(size);
    }
    core::ptr::null_mut()
}

fn main() {
    unsafe {
        let malloc_addr = libc::dlsym(libc::RTLD_DEFAULT, b"malloc\0".as_ptr() as *const i8);
        if malloc_addr.is_null() {
            return;
        }

        MSHookFunction(
            malloc_addr as *mut c_void,
            hook_malloc as *mut c_void,
            core::ptr::addr_of_mut!(ORIG_MALLOC),
        );
    }
}
```

### Example 2: Hooking Game Functions (Android/IL2CPP)

```rust
use substrate::utils::{get_absolute_address, is_library_loaded};
use substrate::MSHookFunction;
use std::ffi::c_void;

static mut OLD_UPDATE: *mut c_void = std::ptr::null_mut();

unsafe extern "C" fn hooked_update(instance: *mut c_void) {
    println!("Game Update() called!");

    // Modify game behavior here

    // Call original
    if !OLD_UPDATE.is_null() {
        let original: extern "C" fn(*mut c_void) =
            std::mem::transmute(OLD_UPDATE);
        original(instance);
    }
}

fn main() {
    unsafe {
        // Wait for library to load
        while !is_library_loaded("libil2cpp.so") {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Hook Update function at offset 0x123456
        let update_addr = get_absolute_address("libil2cpp.so", 0x123456)
            .expect("Failed to find libil2cpp.so");

        MSHookFunction(
            update_addr as *mut c_void,
            hooked_update as *mut c_void,
            &mut OLD_UPDATE
        );

        println!("Update() hooked successfully!");
    }
}
```

### Example 3: Using Helper Macros (C-style)

```rust
use substrate::utils::*;

// Convert hex string to offset
fn hook_with_string_offset() {
    unsafe {
        let offset = string_to_offset("0x123456")
            .expect("Invalid offset");

        let addr = get_absolute_address("libgame.so", offset)
            .expect("Library not found");

        println!("Target address: 0x{:x}", addr);
    }
}
```

### Example 4: Architecture-Specific Hooking

```rust
use substrate;

fn hook_based_on_arch() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // ARM64-specific hooking
        substrate::A64HookFunction(
            target as *mut std::ffi::c_void,
            hook as *mut std::ffi::c_void,
            &mut original as *mut *mut std::ffi::c_void
        );
    }

    #[cfg(target_arch = "arm")]
    unsafe {
        // ARMv7-specific hooking
        substrate::MSHookFunction(
            target as *mut std::ffi::c_void,
            hook as *mut std::ffi::c_void,
            &mut original as *mut *mut std::ffi::c_void
        );
    }
}
```

## API Reference

### Core Functions

#### `MSHookFunction`
```c
void MSHookFunction(void *symbol, void *replace, void **result);
```
Hook a function at the given address. Compatible with original Cydia Substrate.

**Parameters:**
- `symbol`: Target function address to hook
- `replace`: Your hook function address
- `result`: Pointer to store original function (trampoline), can be NULL

#### `A64HookFunction`
```c
void A64HookFunction(void *symbol, void *replace, void **result);
```
ARM64-specific hook function (alias to MSHookFunction on ARM64).

### Utility Functions

#### `findLibrary`
```c
uintptr_t findLibrary(const char *library);
```
Find the base address of a loaded library.

**Example:**
```c
uintptr_t base = findLibrary("libil2cpp.so");
```

#### `getAbsoluteAddress`
```c
uintptr_t getAbsoluteAddress(const char *library, uintptr_t offset);
```
Calculate absolute address from library name and offset.

**Example:**
```c
uintptr_t addr = getAbsoluteAddress("libil2cpp.so", 0x123456);
```

#### `isLibraryLoaded`
```c
bool isLibraryLoaded(const char *library);
```
Check if a library is currently loaded.

**Example:**
```c
if (isLibraryLoaded("libunity.so")) {
    // Library is loaded
}
```

#### `string2Offset`
```c
uintptr_t string2Offset(const char *str);
```
Convert hex string (e.g., "0x123456") to numeric offset.

**Example:**
```c
uintptr_t offset = string2Offset("0x123456");
```

#### `hook`
```c
void hook(void *offset, void *ptr, void **orig);
```
Universal hook function that dispatches to the correct implementation.

### Rust API

```rust
// Safe wrapper for hooking
pub unsafe fn hook_function<T>(
    symbol: *mut T,
    replace: *mut T
) -> Result<*mut T>

// Utility functions
pub fn find_library(library_name: &str) -> Result<usize>
pub fn get_absolute_address(library_name: &str, offset: usize) -> Result<usize>
pub fn is_library_loaded(library_name: &str) -> bool
pub fn string_to_offset(s: &str) -> Result<usize>

// Symbol resolution
pub fn find_symbol_in_process(
    pid: libc::pid_t,
    library: &str,
    symbol: &str
) -> Result<*mut c_void>
```

## Platform Support

| Platform | x86 | x86-64 | ARMv7 | ARM64 | Status |
|----------|-----|--------|-------|-------|--------|
| Linux    | ✅  | ✅     | ✅    | ✅    | Full   |
| Android  | ✅  | ✅     | ✅    | ✅    | Full   |
| macOS    | ❌  | ⚠️     | ❌    | ⚠️    | Untested |

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# With debug logging
cargo build --release --features debug

# Cross-compile for Android ARM64
cargo build --release --target aarch64-linux-android

# Cross-compile for Android ARMv7
cargo build --release --target armv7-linux-androideabi
```

## Advanced Usage

### Custom Error Handling

```rust
use substrate::error::SubstrateError;

match hook_function(target, hook) {
    Ok(original) => println!("Success! Original: {:p}", original),
    Err(SubstrateError::NullPointer) => eprintln!("Null pointer!"),
    Err(SubstrateError::HookFailed(msg)) => eprintln!("Hook failed: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Enable Debug Logging

```rust
use substrate::{set_debug, is_debug};

// Enable debug output
set_debug(true);

// Check if debug is enabled
if is_debug() {
    println!("Debug mode active");
}
```

### Memory Safety

The library uses RAII and safe abstractions internally, but the public API requires `unsafe` due to the nature of function hooking:

```rust
unsafe {
    // All hooking must be in unsafe blocks
    let orig = hook_function(target, replacement)?;

    // Safe to use the trampoline
    let original_fn: extern "C" fn() = std::mem::transmute(orig);
    original_fn();
}
```

## Technical Details

### Architecture-Specific Implementation

- **x86/x86-64**: Full instruction decoder (HDE64), handles RIP-relative addressing
- **ARMv7**: Separate ARM and Thumb mode handlers, PC-relative instruction relocation
- **ARM64**: Complete instruction fixing for all PC-relative types (B, BL, CBZ, LDR, ADR, ADRP, etc.)

### Trampoline Generation

The library automatically:
1. Disassembles instructions at the target
2. Relocates PC-relative instructions
3. Creates a trampoline with original code
4. Installs jump to hook function
5. Returns pointer to trampoline (original function)

### Memory Protection

Automatically handles:
- Memory page protection (mprotect)
- Instruction cache clearing
- Alignment requirements
- Permission restoration

## License

This project is licensed under the GNU Lesser General Public License v3.0 (LGPL-3.0).

Original Cydia Substrate: Copyright (C) 2008-2011 Jay Freeman (saurik)
And64InlineHook: Copyright (C) 2018 Rprop (MIT License)
Rust Implementation: 2024

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Jay Freeman (saurik) - Original Cydia Substrate
- Rprop - And64InlineHook implementation
- Rodroid Mods

## FAQ

**Q: Is this compatible with the original Cydia Substrate?**
A: Yes! It's a drop-in replacement with the same C API.

**Q: Can I use this for game modding?**
A: Yes, it's commonly used for Android game modding (IL2CPP, Unity, Unreal Engine).

**Q: Does it work on rooted devices only?**
A: No, it works on non-rooted devices within your own app's process.

**Q: What about anti-cheat detection?**
A: This is a hooking library. Detection avoidance is your responsibility.

**Q: Performance impact?**
A: Minimal - only affects hooked functions, optimized trampolines.

## Support

- [Documentation](https://docs.rs/substrate-rs)
- [Issue Tracker](https://github.com/rodroidmods/cydia-substrate/issues)
- [Crates.io](https://crates.io/crates/substrate-rs)
