/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include "timer.h"
#include "text_display.h"

// The function that we register as a callback
static void job_done (__attribute__ ((unused)) returncode_t status, void *user_data) {
  bool *done = (bool*)user_data;
  *done = true;
}

int main(void) {
  if (driver_exists(DRIVER_NUM_TEXT_DISPLAY)) {
    // Display the text in a synchronous way
    text_display_show_text_sync ("Hello World from the Microbit", 300);

    // Display the text in an asynchronous way
    bool done = false;
    text_display_set_done_callback (job_done, &done);
    if (text_display_show_text ("Hello World from the Microbit", 300) == RETURNCODE_SUCCESS)
    {
      // verify if we had a callback call
      while (yield_no_wait() == 0 && done == false) {
        // Display . until the we get a callback
        printf (".");
        // Flushing is required as otherwise printf will only send the text
        // to the kernel when it prints a newline.
        fflush (stdout);
        delay_ms (1000);
      }
    }
  } else {
    printf ("Error: the text_display driver is not present\n");
  }
  return 0;
}
