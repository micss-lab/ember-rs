#include <cstdint>
#include <cstddef>

void read_chars_from_uart(uint8_t* buffer, size_t max_len) {
  size_t idx = 0;
  memset(buffer, 0, max_len);
  
  while (idx < max_len - 1) {
    if (Serial.available()) {
      char c = Serial.read();
      if (c == '\n' || c == '\r') {
        break;
      }
      buffer[idx++] = c;
    }
  }
}