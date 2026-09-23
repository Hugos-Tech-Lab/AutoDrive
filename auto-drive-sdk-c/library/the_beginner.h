#ifndef ROBOT_H
#define ROBOT_H

#include <stdint.h>

// Delays the execution.
void delay(uint64_t milliseconds);

// Prints a string to the host console.
void print(const char* text, uint32_t len);

// Sets the onboard LED color using RGB values.
void set_onboard_led_color(uint8_t r, uint8_t g, uint8_t b);

#endif // ROBOT_H
