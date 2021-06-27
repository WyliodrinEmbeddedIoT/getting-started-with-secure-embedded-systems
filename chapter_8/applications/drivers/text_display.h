// Digit Letter Display API

#pragma once

#include "tock.h"

#define TEXT_DISPLAY_DRIVER_NUM 0xa0002

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool done;
    statuscode_t status;
} text_display_status_t;

bool text_display_is_present (void);
int text_display_show_text (const char* text, int display_ms);

#ifdef __cplusplus
}
#endif
