// LED Matrix Text API

// Make sure this file is included only once
#pragma once

#include "tock.h"

#define DRIVER_NUM_LED_MATRIX_TEXT 0xa0003

// Make sure that functions are exported as C functions and not C++
// This prevents the compiler from exporing the functions using
// the C++ name mangling style 
#ifdef __cplusplus
extern "C" {
#endif

// Verifies if the driver is present.
bool led_matrix_text_is_present (void);

// Set the display speed in ms.
bool led_matrix_text_set_speed (unsigned int speed);

#ifdef __cplusplus
}
#endif
