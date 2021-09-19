// Digit Letter Display API

#include "digit_letter_display.h"
#include "tock.h"

bool digit_letter_display_is_present (void) {
  // send command number 0 to the driver
  syscall_return_t ret = command (DRIVER_NUM_DIGIT_LETTER_DISPLAY, 0, 0, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    // if the driver returns SUCCESS, that mmeans that the driver is present
    return true;
  } else {
    return false;
  }
}

bool digit_letter_display_show_character (char digit_or_letter) {
  // send command number 1 to the driver with argument 1 (r2) set 
  // to the digit or letter to display
  syscall_return_t ret = command (DRIVER_NUM_DIGIT_LETTER_DISPLAY, 1, digit_or_letter, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}
