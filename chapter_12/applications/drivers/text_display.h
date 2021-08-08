// Text Display API

#pragma once

#include <tock.h>

#define DISPLAY_BUFFER_LEN 64

#ifdef __cplusplus
extern "C" {
#endif

// verify of the service is present
bool display_text_is_present (void);

// display a test
int display_text (const char *buffer);

#ifdef __cplusplus
}
#endif
