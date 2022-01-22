/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include "timer.h"
#include "digit_letter_display.h"

const char *DISPLAY_TEXT = "MicroBit";

int main(void) {
  // verify if the driver is present
  // if (driver_exists(DRIVER_NUM_DIGIT_LETTER_DISPLAY)) {
  if (digit_letter_display_is_present()) {

    // display all digits
    printf ("Displaying digits\n");
    for (unsigned char digit='0'; digit <= '9'; digit++) {
      printf ("  %c\n", digit);
      // display a digit
      digit_letter_display_show_character (digit);
      // wait for 500 ms
      delay_ms (500);
    }

    // display all letters
    printf ("Displaying letters\n");
    for (unsigned char letter='A'; letter <= 'Z'; letter++) {
      printf ("  %c\n", letter);
      // display a letter
      digit_letter_display_show_character (letter);
      // wait for 500 ms
      delay_ms (500);
    }

    // display text
    for (unsigned int index = 0; index < strlen (DISPLAY_TEXT); index++) {
      digit_letter_display_show_character (DISPLAY_TEXT[index]);
      delay_ms (500);
    }
  } else {
    printf ("Error: the DigitLetterDisplay Syscall Capsule is not present\n");
  }
  return 0;
}
