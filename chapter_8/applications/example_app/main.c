/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include "timer.h"
#include "text_display.h"

static void job_done (__attribute__ ((unused)) statuscode_t status, void *user_data) {
  bool *done = (bool*)user_data;
  *done = true;
}

int main(void) {
  if (text_display_is_present()) {
    // display the text in a synchronous way
    text_display_show_text_sync ("Hello World from the Microbit", 300);

    // display the text in an asynchronous way
    bool done = false;
    text_display_set_done_callback (job_done, &done);
    if (text_display_show_text ("Hello World from the Microbit", 300) == TOCK_STATUSCODE_SUCCESS)
    {
      while (yield_no_wait() == 0 && done == false) {
        printf (".");
        fflush (stdout);
        delay_ms (1000);
      }
    }
  } else {
    printf ("Error: the text_display driver is not present\n");
  }
  return 0;
}
