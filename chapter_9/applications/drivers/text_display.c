// Text Display API

#include "text_display.h"
#include "tock.h"

// used by the asynchronous API
static text_display_done_t *done_callback = NULL;
static void * done_callback_args = NULL;

// used by the synchronous API
typedef struct {
  bool done;
  statuscode_t status;
} text_display_status_t;

// As the command function is used in several places
// we make a wrapper that automatically adds the driver
// number
static syscall_return_t text_display_command(uint32_t command_number, int arg1, int arg2) {
  return command (DRIVER_NUM_TEXT_DISPLAY, command_number, arg1, arg2);
}

// As the allow_readonly function is used in several places
// we make a wrapper that automatically adds the driver
// number
static allow_ro_return_t text_display_allow (uint32_t allow_number, const void* ptr, size_t size) {
  return allow_readonly (DRIVER_NUM_TEXT_DISPLAY, allow_number, ptr, size);
}

// As the subscribe function is used in several places
// we make a wrapper that automatically adds the driver
// number
static subscribe_return_t text_display_subscribe (uint32_t subscribe_number, subscribe_upcall upcall, void* userdata) {
  return subscribe (DRIVER_NUM_TEXT_DISPLAY, subscribe_number, upcall, userdata);
}

bool text_display_is_present (void) {
  // send command number 0 to the driver
  syscall_return_t ret = text_display_command (0, 0, 0);
  if (ret.type == TOCK_SYSCALL_SUCCESS) {
    // if the driver returns SUCCESS, that mmeans that the driver is present
    return true;
  }  else{
    return false;
  }
}

/********* Asynchronous API **********/

// A process will use this funtion to set a callback function
// to be called when a display action is done.
void text_display_set_done_callback (text_display_done_t callback, void *callback_args) {
  done_callback      = callback;
  done_callback_args = callback_args;
}

// The library registers this function with the driver for the asynchronous calls.
// The driver will call this function when a display action is done.
///
// We use this aproach to make sure that the shared buffer 
// is unallowed each time when an action is done.
static void text_displayed (int status, __attribute__ ((unused)) int unused2, __attribute__ (
                              (unused)) int unused3, __attribute__ ((unused)) void *user_data) {
  // Unallow the buffer so that we can access it.
  text_display_allow (0, NULL, 0);
  // Unsubscribe as we are not waiting any other action
  text_display_subscribe (0, NULL, NULL);
  // Verify if the process has registered a callback
  if (done_callback != NULL) {
    // Call the process callback providing
    (*done_callback)(tock_status_to_returncode(status), done_callback_args);
  }
}

// Display a text
returncode_t text_display_show_text (const char* text, unsigned int display_ms) {
  if (text == NULL) {
    return RETURNCODE_EINVAL;
  }
  // Allow the buffer with the driver
  allow_ro_return_t allow_ret = text_display_allow (0, text, strlen (text));
  if (allow_ret.success) {
    // Subscribe to the display finished event using the library's function
    // This library function will in turn call the function that the process
    // has registered.
    subscribe_return_t subscribe_ret = text_display_subscribe (0, text_displayed, NULL);
    if (subscribe_ret.success) {
      // Send command 1 to the driver
      syscall_return_t ret = text_display_command (1, strlen (text), display_ms);
      if (ret.type == TOCK_SYSCALL_SUCCESS) {
        return RETURNCODE_SUCCESS;
      } else {
        // There was an error and the display action could not be started
        
        // Unallow the buffer
        text_display_allow (0, NULL, 0);

        // Unsubscribe
        text_display_subscribe (0, NULL, NULL);

        // Return an error to the process.
        return tock_status_to_returncode(ret.data[0]);
      }
    } else {
      // There was an error an we were not able to subscribe to the
      // driver. We cannot ask the driver to display as it will 
      // not be able to notify us when it finishes.
      
      // Unallow the buffer
      text_display_allow (0, NULL, 0);

      // Return an error to the process.
      return tock_status_to_returncode(subscribe_ret.status);
    }
  } else {
    // We could not allow the buffer with the driver.

    // Return an error to the process.
    return tock_status_to_returncode(allow_ret.status);
  }
}

/******* Synchronous API *********/

// The library registers this function with the driver for the synchronous calls.
// The driver will call this function when a display action is done.
//
// In thsi case, we do not have to call a process callback, as the process
// still waits for the synchronous function call it made to finish.
// The user data that the function receives is a pointer to a boo value
// that the synchronous *text_display_show_text_sync* function verifies
// when yield returns.
static void text_displayed_sync (returncode_t status, void *user_data) {
  text_display_status_t *display_status = (text_display_status_t*)user_data;
  // set the bool value to true
  display_status->done   = true;
  // store the error
  display_status->status = status;
}

// Display a text and wait for it to finish
returncode_t text_display_show_text_sync (const char* text, unsigned int display_ms) {
  text_display_status_t display_status;
  display_status.done   = false;
  display_status.status = 0;

  // Register the callback with the driver
  text_display_set_done_callback (text_displayed_sync, &display_status);

  // Use the asychronous API to display the text
  returncode_t ret = text_display_show_text (text, display_ms);

  if (ret == RETURNCODE_SUCCESS) {
    // If the display has started, wait to our callback function 
    // to be called
    // yield_for is actually
    /* 
      while (!display_status.done) {
        yield ();
      }
    */
    // From the kernel's point of view, this goes like this:
    //  1. yield asks the kernel to yield the process until an upcall is ready
    //  2. when an upcall is ready, the kernel call it by replacing the yield function
    //  3. when the upcall returns, exection continues as if yield had returned
    yield_for (&display_status.done);
    // Return the status to the process
    return tock_status_to_returncode(display_status.status);
  } else {
    // Return the error to the process
    return ret;
  }
}
