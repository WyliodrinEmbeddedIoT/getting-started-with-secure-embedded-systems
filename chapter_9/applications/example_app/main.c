/* vim: set sw=2 expandtab tw=80: */

#include "led_matrix_text.h"
#include "text_screen.h"
#include "timer.h"
#include <stdio.h>

#define SCREEN_BUFFER_SIZE 50

int main(void) {
  if (driver_exists(DRIVER_NUM_TEXT_SCREEN)) {
    if (text_screen_init(SCREEN_BUFFER_SIZE) == RETURNCODE_SUCCESS) {
      char *buffer = (char*)text_screen_buffer ();
      strcpy (buffer, "Hello World from the Microbit");
      text_screen_set_cursor (0, 0);
      text_screen_write (strlen (buffer));
      if (driver_exists (DRIVER_NUM_LED_MATRIX_TEXT)) {
        printf ("Setting speed to 500\n");
        led_matrix_text_set_speed (500);
      }
    } else {
      printf ("Error: failed to initialize text screen\n");
    }
  } else {
    printf ("Error: text screen driver is not present\n");
  }
  return 0;
}
