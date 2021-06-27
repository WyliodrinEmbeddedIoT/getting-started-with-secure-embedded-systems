// Digit Letter Display API

#include "text_display.h"
#include "tock.h"

static void text_displayed (int status, __attribute__ ((unused)) int unused2, __attribute__ ((unused)) int unused3, void *user_data) {
  text_display_status_t *display_status = (text_display_status_t*)user_data;
  display_status->done = true;
  display_status->status = status;
}

static syscall_return_t text_display_command(uint32_t command_number, int arg1, int arg2) {
  return command (TEXT_DISPLAY_DRIVER_NUM, command_number, arg1, arg2);
}

static allow_ro_return_t text_display_allow (uint32_t allow_number, const void* ptr, size_t size) {
  return allow_readonly (TEXT_DISPLAY_DRIVER_NUM, allow_number, ptr, size);
}

static subscribe_return_t text_display_subscribe (uint32_t subscribe_number, subscribe_upcall upcall, void* userdata) {
  return subscribe (TEXT_DISPLAY_DRIVER_NUM, subscribe_number, upcall, userdata);
}

bool text_display_is_present (void) {
  syscall_return_t ret = text_display_command (0, 0, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  } else {
    return false;
  }
}

int text_display_show_text (const char* text, int display_ms) {
  // allow the buffer
  allow_ro_return_t allow_ret = text_display_allow (0, text, strlen (text));
  if (allow_ret.success)
  {
    // structure used to store status
    text_display_status_t display_status;
    display_status.done = false;
    display_status.status = 0;

    // subscibe to the display finished event
    subscribe_return_t subscribe_ret = text_display_subscribe (0, text_displayed, &display_status);

    if (subscribe_ret.success) {

      // execute command
      syscall_return_t ret = text_display_command (1, strlen (text), display_ms);
      if (ret.type == TOCK_SYSCALL_SUCCESS) {
        // wait for displaying the text
        while (display_status.done == false) {
          yield ();
        }
      } 

      // unallow the buffer
      text_display_allow (0, NULL, 0);

      // unsubscribe
      text_display_subscribe (0, NULL, NULL);

      if (ret.type == TOCK_SYSCALL_SUCCESS) {
        return display_status.status;
      }
      else
      {
        return tock_status_to_returncode (ret.data[0]);
      }
    }
    else
    {
      // unallow the buffer
      text_display_allow (0, NULL, 0);
      return tock_status_to_returncode (subscribe_ret.status);
    }
  }
  else
  {
    return tock_status_to_returncode (allow_ret.status);
  }
}
