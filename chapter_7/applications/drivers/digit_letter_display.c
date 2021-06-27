// Digit Letter Display API

#include "digit_letter_display.h"
#include "tock.h"

bool digit_letter_display_is_present (void) {
  syscall_return_t ret = command (DIGIT_LETTER_DISPLAY_DRIVER_NUM, 0, 0, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}

bool digit_letter_display_show_character (char digit_or_letter) {
  syscall_return_t ret = command (DIGIT_LETTER_DISPLAY_DRIVER_NUM, 1, digit_or_letter, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}
