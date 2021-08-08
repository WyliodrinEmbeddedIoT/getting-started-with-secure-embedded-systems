// Text Display API

#include "text_display.h"
#include <tock.h>
#include <ipc.h>

char display_buffer[DISPLAY_BUFFER_LEN] __attribute__((aligned(64)));
int text_display_service = -1;

static void ipc_callback(__attribute__((unused)) int pid, __attribute__((unused)) int len, __attribute__((unused)) int buf, void* ud) {
  bool *done = (bool*)ud;
  *done = true;
}

bool display_text_is_present (void) {
  // verifies if the service is present and 
  // and registers its id
  return ipc_discover ("text_display.service", &text_display_service) == RETURNCODE_SUCCESS;
}

int display_text (const char *buffer) {
  int ret = RETURNCODE_SUCCESS;
  bool done = false;

  // if the service ID has not yet been registered,
  // try to register it
  if (text_display_service == -1) display_text_is_present();

  // if the service is present, display the text
  if (text_display_service)
  {
    // copy the text into the shared buffer
    strncpy (display_buffer, buffer, DISPLAY_BUFFER_LEN);

    // share the buffer with the service
    // the buffer should not be accessed anymore
    ret = ipc_share(text_display_service, display_buffer, DISPLAY_BUFFER_LEN);
    if (ret == RETURNCODE_SUCCESS) {
      // register the service client callback
      // to get notified when the service buffer 
      // has finished copying the data from the buffer
      ret = ipc_register_client_callback(text_display_service, ipc_callback, &done);
      if (ret == RETURNCODE_SUCCESS)
      {
        // notify the service to display the text
        ret = ipc_notify_service(text_display_service);
        // wait for the service to copy the text from the shared
        // buffer to its own buffer
        if (ret == RETURNCODE_SUCCESS) {
          yield_for(&done);
        }
      }
      // stop sharing the buffer so that is becomes
      // accesible to the application
      ret = ipc_share(text_display_service, NULL, 0);
    }
  }
  return ret;
}