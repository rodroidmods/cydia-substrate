/**
 * C Example - Using Substrate for Function Hooking
 *
 * Compile:
 *   gcc example.c -o example -L../target/release -lsubstrate -ldl
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./example
 */

#include "../substrate.h"
#include <stdio.h>
#include <stdlib.h>

/* Original function pointer */
void (*old_FixedUpdate)(void *instance) = NULL;

/* Hook function */
void hooked_FixedUpdate(void *instance) {
    printf("[Hook] FixedUpdate called! Instance: %p\n", instance);

    /* Call original if available */
    if (old_FixedUpdate) {
        old_FixedUpdate(instance);
    }
}

int main() {
    printf("=== Substrate C Example ===\n\n");

    /* Example 1: Using getAbsoluteAddress */
    printf("1. Hook using library + offset:\n");

    uintptr_t addr = getAbsoluteAddress("libil2cpp.so", 0x123456);
    if (addr != 0) {
        printf("   Target address: 0x%lx\n", addr);

        MSHookFunction(
            (void *)addr,
            (void *)hooked_FixedUpdate,
            (void **)&old_FixedUpdate
        );

        printf("   ✓ Hook installed!\n");
        printf("   Original function: %p\n\n", old_FixedUpdate);
    } else {
        printf("   ✗ Library not found or invalid offset\n\n");
    }

    /* Example 2: Check if library is loaded */
    printf("2. Check library status:\n");

    if (isLibraryLoaded("libc.so.6")) {
        printf("   ✓ libc.so.6 is loaded\n");

        uintptr_t libc_base = findLibrary("libc.so.6");
        printf("   Base address: 0x%lx\n\n", libc_base);
    } else {
        printf("   ✗ libc.so.6 not loaded\n\n");
    }

    /* Example 3: Parse hex offset */
    printf("3. Parse hex string:\n");

    uintptr_t offset = string2Offset("0xABCDEF");
    printf("   \"0xABCDEF\" = 0x%lX\n\n", offset);

    /* Example 4: Using the hook() helper */
    printf("4. Using hook() helper:\n");
    printf("   void (*old_func)(void);\n");
    printf("   hook(target_addr, hook_func, (void**)&old_func);\n\n");

    printf("=== Example Complete ===\n");
    printf("\nNote: This example uses placeholder addresses.\n");
    printf("Replace with actual function addresses for real usage.\n");

    return 0;
}
