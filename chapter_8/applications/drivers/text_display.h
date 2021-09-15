// Text Display API

#pragma once

#include "tock.h"

#define DRIVER_NUM_TEXT_DISPLAY 0xa0002

#ifdef __cplusplus
extern "C" {
#endif

typedef void (text_display_done_t)(returncode_t, void *user_data);

// Presence
bool text_display_is_present (void);

// Asynchronous API
void text_display_set_done_callback (text_display_done_t callback, void *callback_args);
returncode_t text_display_show_text (const char* text, unsigned int display_ms);

// Synchronous API
returncode_t text_display_show_text_sync (const char* text, unsigned int display_ms);

#ifdef __cplusplus
}
#endif
