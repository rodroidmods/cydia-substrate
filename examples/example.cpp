/**
 * C++ Example - Using Substrate for Function Hooking
 *
 * Compile:
 *   g++ example.cpp -o example -L../target/release -lsubstrate -ldl -std=c++11
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./example
 */

#include "../substrate.h"
#include <iostream>
#include <thread>
#include <chrono>

using namespace std;

/* Original function pointers */
void (*old_Update)(void *instance) = nullptr;
void (*old_FixedUpdate)(void *instance) = nullptr;

/* Hook functions */
void hooked_Update(void *instance) {
    cout << "[Hook] Update() called - Instance: " << instance << endl;

    if (old_Update) {
        old_Update(instance);
    }
}

void hooked_FixedUpdate(void *instance) {
    cout << "[Hook] FixedUpdate() called - Instance: " << instance << endl;

    if (old_FixedUpdate) {
        old_FixedUpdate(instance);
    }
}

void wait_for_library(const char *lib_name, int timeout_seconds = 30) {
    cout << "Waiting for " << lib_name << " to load..." << endl;

    for (int i = 0; i < timeout_seconds * 10; i++) {
        if (isLibraryLoaded(lib_name)) {
            cout << "✓ " << lib_name << " loaded!" << endl;
            return;
        }
        this_thread::sleep_for(chrono::milliseconds(100));
    }

    cout << "✗ Timeout waiting for " << lib_name << endl;
}

int main() {
    cout << "=== Substrate C++ Example ===" << endl << endl;

    const char *library = "libil2cpp.so";

    /* Check if library is loaded */
    if (isLibraryLoaded(library)) {
        cout << "✓ " << library << " is already loaded" << endl;
    } else {
        cout << "Library not loaded, waiting..." << endl;
        wait_for_library(library);
    }

    /* Get library base address */
    uintptr_t base = findLibrary(library);
    if (base != 0) {
        cout << "Library base address: 0x" << hex << base << dec << endl << endl;
    }

    /* Hook Update function */
    cout << "1. Hooking Update()..." << endl;

    uintptr_t update_offset = string2Offset("0x123456");
    uintptr_t update_addr = getAbsoluteAddress(library, update_offset);

    if (update_addr != 0) {
        cout << "   Target: 0x" << hex << update_addr << dec << endl;

        MSHookFunction(
            reinterpret_cast<void *>(update_addr),
            reinterpret_cast<void *>(hooked_Update),
            reinterpret_cast<void **>(&old_Update)
        );

        if (old_Update) {
            cout << "   ✓ Update() hooked successfully!" << endl;
            cout << "   Original: " << old_Update << endl << endl;
        }
    }

    /* Hook FixedUpdate function */
    cout << "2. Hooking FixedUpdate()..." << endl;

    uintptr_t fixed_offset = string2Offset("0x789ABC");
    uintptr_t fixed_addr = getAbsoluteAddress(library, fixed_offset);

    if (fixed_addr != 0) {
        cout << "   Target: 0x" << hex << fixed_addr << dec << endl;

        /* Using the C++ HOOK macro */
        HOOK(library, "0x789ABC", hooked_FixedUpdate, old_FixedUpdate);

        if (old_FixedUpdate) {
            cout << "   ✓ FixedUpdate() hooked successfully!" << endl;
            cout << "   Original: " << old_FixedUpdate << endl << endl;
        }
    }

    /* Example with architecture detection */
    cout << "3. Architecture detection:" << endl;

#ifdef __aarch64__
    cout << "   Running on ARM64 (AArch64)" << endl;
    cout << "   Using A64HookFunction API" << endl;
#elif defined(__arm__)
    cout << "   Running on ARMv7 (32-bit)" << endl;
    cout << "   Using MSHookFunction API" << endl;
#elif defined(__x86_64__)
    cout << "   Running on x86-64" << endl;
    cout << "   Using MSHookFunction API" << endl;
#elif defined(__i386__)
    cout << "   Running on x86 (32-bit)" << endl;
    cout << "   Using MSHookFunction API" << endl;
#else
    cout << "   Unknown architecture" << endl;
#endif

    cout << endl << "=== Hooks Installed ===" << endl;
    cout << "The target functions will now call your hooks." << endl;
    cout << endl << "Note: Replace offset values with real ones from your target!" << endl;

    return 0;
}
