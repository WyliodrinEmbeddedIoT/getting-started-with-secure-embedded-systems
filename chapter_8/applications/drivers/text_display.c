// Text Display API

#include "text_display.h"
#include "tock.h"

// asynchronous
static text_display_done_t *done_callback = NULL;
static void * done_callback_args = NULL;

// synchronous
typedef struct {
  bool done;
  statuscode_t status;
} text_display_status_t;

static syscall_return_t text_display_command(uint32_t command_number, int arg1, int arg2) {
  return command (DRIVER_NUM_TEXT_DISPLAY, command_number, arg1, arg2);
}

static allow_ro_return_t text_display_allow (uint32_t allow_number, const void* ptr, size_t size) {
  return allow_readonly (DRIVER_NUM_TEXT_DISPLAY, allow_number, ptr, size);
}

static subscribe_return_t text_display_subscribe (uint32_t subscribe_number, subscribe_upcall upcall, void* userdata) {
  return subscribe (DRIVER_NUM_TEXT_DISPLAY, subscribe_number, upcall, userdata);
}

bool text_display_is_present (void) {
  syscall_return_t ret = text_display_command (0, 0, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    return true;
  }  else{
    return false;
  }
}

// Asynchronous API

void text_display_set_done_callback (text_display_done_t callback, void *callback_args) {
  done_callback      = callback;
  done_callback_args = callback_args;
}

static void text_displayed (int status, __attribute__ ((unused)) int unused2, __attribute__ (
                              (unused)) int unused3, __attribute__ ((unused)) void *user_data) {
  text_display_allow (0, NULL, 0);
  text_display_subscribe (0, NULL, NULL);
  if (done_callback != NULL) {
    (*done_callback)(tock_status_to_returncode(status), done_callback_args);
  }
}

returncode_t text_display_show_text (const char* text, unsigned int display_ms) {
  if (text == NULL) {
    return RETURNCODE_EINVAL;
  }
  // allow the buffer
  allow_ro_return_t allow_ret = text_display_allow (0, text, strlen (text));
  if (allow_ret.success) {
    // subscribe to the display finished event
    subscribe_return_t subscribe_ret = text_display_subscribe (0, text_displayed, NULL);
    if (subscribe_ret.success) {
      // execute command
      syscall_return_t ret = text_display_command (1, strlen (text), display_ms);
      if (ret.type == TOCK_SYSCALL_SUCCESS) {
        return RETURNCODE_SUCCESS;
      } else {
        // unallow the buffer
        text_display_allow (0, NULL, 0);

        // unsubscribe
        text_display_subscribe (0, NULL, NULL);

        return tock_status_to_returncode(ret.data[0]);
      }
    } else {
      // unallow the buffer
      text_display_allow (0, NULL, 0);
      return tock_status_to_returncode(subscribe_ret.status);
    }
  } else {
    return tock_status_to_returncode(allow_ret.status);
  }
}

// Synchronous API

static void text_displayed_sync (statuscode_t status, void *user_data) {
  text_display_status_t *display_status = (text_display_status_t*)user_data;
  display_status->done   = true;
  display_status->status = status;
}

returncode_t text_display_show_text_sync (const char* text, unsigned int display_ms) {
  text_display_status_t display_status;
  display_status.done   = false;
  display_status.status = 0;

  text_display_set_done_callback (text_displayed_sync, &display_status);
  returncode_t ret = text_display_show_text (text, display_ms);

  if (ret == RETURNCODE_SUCCESS) {
    yield_for (&display_status.done);
    return tock_status_to_returncode(display_status.status);
  } else {
    return ret;
  }
}
