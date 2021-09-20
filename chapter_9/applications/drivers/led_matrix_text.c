// Digit Letter Display API

#include "led_matrix_text.h"
#include "tock.h"

bool led_matrix_text_is_present (void) {
  // send command number 0 to the driver
  syscall_return_t ret = command (DRIVER_NUM_LED_MATRIX_TEXT, 0, 0, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}

bool led_matrix_text_set_speed (unsigned int speed) {
  // Send command number 1 to the driver with argument 1 (r2) set 
  // to the speed in ms at which to display.
  syscall_return_t ret = command (DRIVER_NUM_LED_MATRIX_TEXT, 1, speed, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}
