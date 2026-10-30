#ifndef THE_BEGINNER_H
#define THE_BEGINNER_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ---------------------------------------------------------
// FFI Imports (Mapped to the "host" module in WebAssembly)
// ---------------------------------------------------------

/** takes u8 so we don't wait too long (i.e. 255ms max) between cancellation checks */
__attribute__((import_module("host"), import_name("delay")))
extern void delay(uint8_t milliseconds);

/** prints from memory */
__attribute__((import_module("host"), import_name("print")))
extern void print(const char * ptr, uint32_t len);

/** Sets the onboard LED color */
__attribute__((import_module("host"), import_name("set_onboard_led_color")))
extern void set_onboard_led_color(uint8_t r, uint8_t g, uint8_t b);

/** Sets motor A speed */
__attribute__((import_module("host"), import_name("set_motor_a_speed")))
extern void set_motor_a_speed(int8_t speed);

#ifdef __cplusplus
}
#endif

#endif // THE_BEGINNER_H
