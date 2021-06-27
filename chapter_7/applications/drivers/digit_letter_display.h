// Digit Letter Display API

#pragma once

#include "tock.h"

#define DIGIT_LETTER_DISPLAY_DRIVER_NUM 0xa0001

#ifdef __cplusplus
extern "C" {
#endif

bool digit_letter_display_is_present (void);
bool digit_letter_display_show_character (char digit_or_letter);

#ifdef __cplusplus
}
#endif
