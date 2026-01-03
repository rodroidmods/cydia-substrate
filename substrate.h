/**
 * Substrate - Powerful Code Injection Platform
 *
 * Rust implementation of Cydia Substrate and And64InlineHook
 *
 * Copyright (C) 2024 Substrate Contributors
 * Licensed under LGPL-3.0
 *
 * Original Cydia Substrate: Copyright (C) 2008-2011 Jay Freeman (saurik)
 * Original And64InlineHook: Copyright (C) 2018 Rprop
 */

#ifndef SUBSTRATE_H_
#define SUBSTRATE_H_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque type for image references
 */
typedef const void *MSImageRef;

/**
 * Global debug flag - set to true to enable debug logging
 */
extern bool MSDebug;

/**
 * Hook a function at the given address
 *
 * @param symbol  Target function address to hook
 * @param replace Your hook function address
 * @param result  Pointer to store original function (trampoline), can be NULL
 *
 * Example:
 *   void (*old_func)(void);
 *   MSHookFunction((void*)0x12345, (void*)my_hook, (void**)&old_func);
 */
void MSHookFunction(void *symbol, void *replace, void **result);

/**
 * ARM64-specific hook function (alias to MSHookFunction on ARM64)
 *
 * @param symbol  Target function address to hook
 * @param replace Your hook function address
 * @param result  Pointer to store original function (trampoline), can be NULL
 *
 * Example:
 *   void (*old_func)(void);
 *   A64HookFunction((void*)0x12345, (void*)my_hook, (void**)&old_func);
 */
void A64HookFunction(void *symbol, void *replace, void **result);

/**
 * Find a symbol in an image (stub - not fully implemented)
 *
 * @param image Image reference
 * @param name  Symbol name to find
 * @return Pointer to symbol, or NULL if not found
 */
void *MSFindSymbol(MSImageRef image, const char *name);

/**
 * Get image reference by filename (stub - not fully implemented)
 *
 * @param file Filename to look up
 * @return Image reference, or NULL if not found
 */
MSImageRef MSGetImageByName(const char *file);

/**
 * Hook into another process (stub - not implemented)
 *
 * @param pid     Target process ID
 * @param library Library to inject
 * @return true on success, false on failure
 */
bool MSHookProcess(int pid, const char *library);

/**
 * Find the base address of a loaded library
 *
 * @param library Library name (e.g., "libil2cpp.so")
 * @return Base address, or 0 if not found
 *
 * Example:
 *   uintptr_t base = findLibrary("libil2cpp.so");
 *   if (base != 0) {
 *       printf("Library loaded at: 0x%lx\n", base);
 *   }
 */
uintptr_t findLibrary(const char *library);

/**
 * Calculate absolute address from library name and offset
 *
 * @param library Library name (e.g., "libil2cpp.so")
 * @param offset  Offset within the library
 * @return Absolute address, or 0 on error
 *
 * Example:
 *   uintptr_t addr = getAbsoluteAddress("libil2cpp.so", 0x123456);
 *   MSHookFunction((void*)addr, (void*)my_hook, (void**)&original);
 */
uintptr_t getAbsoluteAddress(const char *library, uintptr_t offset);

/**
 * Check if a library is currently loaded
 *
 * @param library Library name (e.g., "libunity.so")
 * @return true if loaded, false otherwise
 *
 * Example:
 *   if (isLibraryLoaded("libunity.so")) {
 *       // Library is loaded, safe to hook
 *   }
 */
bool isLibraryLoaded(const char *library);

/**
 * Convert hex string to numeric offset
 *
 * @param str Hex string (e.g., "0x123456" or "123ABC")
 * @return Numeric offset value, or 0 on parse error
 *
 * Example:
 *   uintptr_t offset = string2Offset("0x123456");
 */
uintptr_t string2Offset(const char *str);

/**
 * Universal hook function that dispatches to the correct implementation
 *
 * @param offset Address to hook
 * @param ptr    Hook function
 * @param orig   Pointer to store original function (can be NULL)
 *
 * Example:
 *   void (*old_update)(void*);
 *   hook((void*)addr, (void*)hooked_update, (void**)&old_update);
 */
void hook(void *offset, void *ptr, void **orig);

#ifdef __cplusplus
}
#endif

/**
 * C++ Convenience Macros (optional)
 */
#ifdef __cplusplus

/**
 * Hook a function by library and offset
 *
 * Usage:
 *   HOOK("libgame.so", "0x123456", my_hook, original);
 */
#define HOOK(lib, offset, ptr, orig) \
    hook((void*)getAbsoluteAddress(lib, string2Offset(offset)), (void*)ptr, (void**)&orig)

/**
 * Hook a function without saving original
 *
 * Usage:
 *   HOOK_NO_ORIG("libgame.so", "0x123456", my_hook);
 */
#define HOOK_NO_ORIG(lib, offset, ptr) \
    hook((void*)getAbsoluteAddress(lib, string2Offset(offset)), (void*)ptr, NULL)

#endif // __cplusplus

#endif // SUBSTRATE_H_
