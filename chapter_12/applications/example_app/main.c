/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include <timer.h>
#include "text_display.h"

int main(void) {
  if (display_text_is_present())
  {
    display_text ("Hello World from the Microbit");
  } else {
    printf ("Error: the text_display.driver service is not present\n");
  }
  return 0;
}
