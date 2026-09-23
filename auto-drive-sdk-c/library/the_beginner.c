#include "the_beginner.h"

// ---------------------------------------------------------
// FFI Imports (Mapped to the "host" module in WebAssembly)
// ---------------------------------------------------------

__attribute__((import_module("host"), import_name("delay")))
extern void delay(uint8_t milliseconds);

__attribute__((import_module("host"), import_name("print")))
extern void print(const char* ptr, uint32_t len);

__attribute__((import_module("host"), import_name("set_onboard_led_color")))
extern void set_onboard_led_color(uint8_t r, uint8_t g, uint8_t b);

// ---------------------------------------------------------
// Public API Exports
// ---------------------------------------------------------

// The export_name attribute ensures the WASM file exposes these exact function names
#define WASM_EXPORT __attribute__((export_name(#__VA_ARGS__)))

__attribute__((export_name("delay")))
void robot_delay(uint64_t milliseconds) {
    while (milliseconds > 255) {
        delay(255);
        milliseconds -= 255;
    }

    if (milliseconds > 0) {
        delay((uint8_t)milliseconds);
    }
}

__attribute__((export_name("print")))
void robot_print(const char* text, uint32_t len) {
    print(text, len);
}

__attribute__((export_name("set_onboard_led_color")))
void robot_set_onboard_led_color(uint8_t r, uint8_t g, uint8_t b) {
    set_onboard_led_color(r, g, b);
}