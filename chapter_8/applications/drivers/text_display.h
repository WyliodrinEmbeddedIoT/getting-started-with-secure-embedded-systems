// Text Display API

#pragma once

#include "tock.h"

#define TEXT_DISPLAY_DRIVER_NUM 0xa0002

#ifdef __cplusplus
extern "C" {
#endif

typedef void (text_display_done_t)(statuscode_t, void *user_data);

// Presence
bool text_display_is_present (void);

// Asynchronous API
void text_display_set_done_callback (text_display_done_t callback, void *callback_args);
statuscode_t text_display_show_text (const char* text, unsigned int display_ms);

// Synchronous API
statuscode_t text_display_show_text_sync (const char* text, unsigned int display_ms);

#ifdef __cplusplus
}
#endif
