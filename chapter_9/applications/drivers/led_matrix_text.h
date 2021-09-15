// LED Matrix Text API

#pragma once

#include "tock.h"

#define DRIVER_NUM_LED_MATRIX_TEXT 0xa0003

#ifdef __cplusplus
extern "C" {
#endif

bool led_matrix_text_is_present (void);
bool led_matrix_text_set_speed (unsigned int speed);

#ifdef __cplusplus
}
#endif
