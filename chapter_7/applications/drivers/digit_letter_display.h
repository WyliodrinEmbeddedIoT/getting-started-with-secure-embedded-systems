// Digit Letter Display API

// Make sure this file is included only once
#pragma once

#include "tock.h"

#define DRIVER_NUM_DIGIT_LETTER_DISPLAY 0xa0001

// Make sure that functions are exported as C functions and not C++
// This prevents the compiler from exporing the functions using
// the C++ name mangling style 
#ifdef __cplusplus
extern "C" {
#endif

// Verifies if the driver is present
bool digit_letter_display_is_present (void);

// Displays a letter or a digit
bool digit_letter_display_show_character (char digit_or_letter);

#ifdef __cplusplus
}
#endif
