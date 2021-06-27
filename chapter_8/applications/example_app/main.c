/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include "timer.h"
#include "text_display.h"

int main(void) {
  if (text_display_is_present()) {
    text_display_show_text ("Hello World from the Microbit", 300);
  } else {
    printf ("Error: the text_display driver is not present\n");
  }
  return 0;
}
