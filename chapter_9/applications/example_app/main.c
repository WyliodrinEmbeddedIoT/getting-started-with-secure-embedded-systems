/* vim: set sw=2 expandtab tw=80: */

#include "led_matrix_text.h"
#include "text_screen.h"
#include "timer.h"
#include <stdio.h>

#define SCREEN_BUFFER_SIZE 50

int main(void) {
  if (text_screen_init(SCREEN_BUFFER_SIZE) == TOCK_STATUSCODE_SUCCESS) {
    char *buffer = (char*)text_screen_buffer ();
    strcpy (buffer, "Hello World from the Microbit");
    text_screen_set_cursor (0, 0);
    text_screen_write (strlen (buffer));
    if (led_matrix_text_is_present ()) {
      printf ("Setting speed to 500\n");
      led_matrix_text_set_speed (500);
    }
  } else {
    printf ("Error: failed to initialize text screen\n");
  }
  return 0;
}
