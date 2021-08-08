/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include <timer.h>
#include <ipc.h>

#define DISPLAY_BUFFER_LEN 64

char display_buffer[DISPLAY_BUFFER_LEN] __attribute__((aligned(64)));

static int display_text (int text_display, const char *buffer) {
  int ret = RETURNCODE_SUCCESS;
  ret = ipc_share(text_display, NULL, 0);
  if (ret == RETURNCODE_SUCCESS)
  {
    strncpy (display_buffer, buffer, DISPLAY_BUFFER_LEN);
    ret = ipc_share(text_display, display_buffer, DISPLAY_BUFFER_LEN);
    if (ret == RETURNCODE_SUCCESS) {
      ret = ipc_notify_service(text_display);
    }
  }
  return ret;
}

int main(void) {
  int text_display;

  if (ipc_discover ("text_display.driver", &text_display) == RETURNCODE_SUCCESS)
  {
    display_text (text_display, "Hello World from the Microbit");
  } else {
    printf ("Error: the text_display.driver service is not present\n");
  }
  return 0;
}
