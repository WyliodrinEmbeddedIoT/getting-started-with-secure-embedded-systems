// Text Display API

// Make sure this file is included only once
#pragma once

#include "tock.h"

#define DRIVER_NUM_TEXT_DISPLAY 0xa0002

// Make sure that functions are exported as C functions and not C++
// This prevents the compiler from exporing the functions using
// the C++ name mangling style 
#ifdef __cplusplus
extern "C" {
#endif

typedef void (text_display_done_t)(returncode_t, void *user_data);

// Presence
bool text_display_is_present (void);

/******** Asynchronous API *********/

// Set a callback function to be called when the text display is done.
void text_display_set_done_callback (text_display_done_t callback, void *callback_args);

// Display the text and immediately return
returncode_t text_display_show_text (const char* text, unsigned int display_ms);

/******** Synchronous API **********/

// Display the text and wait until it is done
returncode_t text_display_show_text_sync (const char* text, unsigned int display_ms);

#ifdef __cplusplus
}
#endif
