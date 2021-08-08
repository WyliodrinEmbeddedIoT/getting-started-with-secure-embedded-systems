/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include <ctype.h>
#include <timer.h>
#include <led.h>
#include <ipc.h>

#define NUM_LEDS 25
#define BUFFER_LEN 50

static char BUFFER[BUFFER_LEN];

#define MIN(a,b) (a<b?a:b)

const uint32_t DIGITS[] = {
    // 0
    0b1111110011101011100111111,
    // 1
    0b0010001100001000010001110,
    // 2
    0b1111000001011101000011111,
    // 3
    0b1111000001111100000111110,
    // 4
    0b1000010000101001111100100,
    // 5
    0b1111110000111100000111110,
    // 6
    0b1111110000111111000111111,
    // 7
    0b1111100001000100010000100,
    // 8
    0b1111110001111111000111111,
    // 9
    0b1111110001111110000111111,
};

const uint32_t LETTERS[] = {
    // A
    0b0111010001111111000110001,
    // B
    0b1111110001111101000111111,
    // C
    0b1111110000100001000011111,
    // D
    0b1111010001100011000111110,
    // E
    0b1111110000111101000011111,
    // F
    0b1111110000111101000010000,
    // G
    0b1111110000101111000111111,
    // H
    0b1000110001111111000110001,
    // I
    0b1111100100001000010011111,
    // J
    0b0001100001000011000111111,
    // K
    0b1000110010111001001010001,
    // L
    0b1000010000100001000011111,
    // M
    0b1000111011101011000110001,
    // N
    0b1000111001101011001110001,
    // O
    0b0111010001100011000101110,
    // P
    0b1111010001111101000010000,
    // Q
    0b0111010001100010111000011,
    // R
    0b1111010001111101000110001,
    // S
    0b1111110000111110000111111,
    // T
    0b1111100100001000010000100,
    // U
    0b1000110001100011000111111,
    // V
    0b1000110001010100101000100,
    // W
    0b1000110001101011010101010,
    // X
    0b1000101010001000101010001,
    // Y
    0b1000110001010100010000100,
    // Z
    0b1111100010001000100011111,
};

static void ipc_callback(int pid, int len, int buf, __attribute__((unused)) void* ud) {
  // update the buffer with data from an app
  const char *buffer = (const char *)buf;
  printf ("buffer %p\n", (void*)buf);
  if (buffer != NULL) {
    printf("Recevided display request from process %d\n", pid);
    strncpy (BUFFER, buffer, MIN(BUFFER_LEN, len));
  }
}

static void display_code (uint32_t code) {
  int led_index = 0;
  for (led_index=0; led_index<NUM_LEDS; led_index++) {
    if (((code >> (NUM_LEDS - 1 - led_index)) & 0x1) == 1) {
      led_on(led_index);
    }
    else
    {
      led_off(led_index);
    }
  }
}

static void display(char digit_or_letter) {
  digit_or_letter = toupper(digit_or_letter);
  if (digit_or_letter >= '0' && digit_or_letter <= '9') {
    display_code(DIGITS[digit_or_letter - '0']);
  }
  else if (digit_or_letter >= 'A' && digit_or_letter <= 'Z') {
    display_code(LETTERS[digit_or_letter - 'A']);
  }
}

static void clear(void) {
  int led_index = 0;
  for (led_index=0; led_index<NUM_LEDS; led_index++) {
    led_off(led_index);
  }
}

int main(void) {
  int leds;
  int position = 0;
  int len = 0;

  strcpy (BUFFER, "");

  if (led_count (&leds) == RETURNCODE_SUCCESS) {
    if (leds >= 25) {
      // register the digit_letter.driver service
      ipc_register_service_callback(ipc_callback, NULL);

      // run the service
      while (true) {
        len = strnlen (BUFFER, BUFFER_LEN);
        if (len == 0) {
          position = 0;
          clear ();
        }
        else if (position < len) {
          position = (position + 1) % len;
          display (BUFFER[position]);
        }
        delay_ms(300);
      }
    }
    else
    {
      printf ("digit_letter_driver: Expected 25 LEDs, available %d\n", leds);
    }
  }
  else
  {
    printf ("digit_letter_driver: LEDs driver is not available\n");
  }
  return 0;
}
