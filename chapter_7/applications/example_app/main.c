/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include "timer.h"
#include "digit_letter_display.h"

int main(void) {
  // if (driver_exists(DRIVER_NUM_DIGIT_LETTER_DISPLAY)) {
  if (digit_letter_display_is_present()) {
    printf ("Displaying digits\n");
    for (unsigned char digit='0'; digit <= '9'; digit++) {
      printf ("  %c\n", digit);
      digit_letter_display_show_character (digit);
      delay_ms (500);
    }
    printf ("Displaying letters\n");
    for (unsigned char letter='A'; letter <= 'Z'; letter++) {
      printf ("  %c\n", letter);
      digit_letter_display_show_character (letter);
      delay_ms (500);
    }
    } else {
    printf ("Error: the digit_letter_display driver is not present\n");
  }
  return 0;
}
