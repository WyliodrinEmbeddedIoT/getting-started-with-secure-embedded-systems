// Digit Letter Display API

#include "led_matrix_text.h"
#include "tock.h"

bool led_matrix_text_is_present (void) {
  syscall_return_t ret = command (DRIVER_NUM_LED_MATRIX_TEXT, 0, 0, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}

bool led_matrix_text_set_speed (unsigned int speed) {
  syscall_return_t ret = command (DRIVER_NUM_LED_MATRIX_TEXT, 1, speed, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}
