// #define USE_EMBER // Uncomment this to use the ember library.

#include <cstdint>
#include <cstddef>

#ifdef USE_EMBER
#include "Ember.h"
#endif // USE_EMBER

#include "./common.h"

#ifdef USE_EMBER

std::unique_ptr<ember::Container> container;

#else 

bool door_locked = true;
unsigned long unlocked_at = 0;
bool object_detected = false;

const char LOCK_PASSWORD[5] = "1234";

#endif // USE_EMBER

/******************************************
  Helper Functions
******************************************/
#ifndef USE_EMBER

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

void unlock() {
  Serial.println("Unlocking door, enter password:");
  
  uint8_t password[25];
  read_chars_from_uart(password, 25);
  
  Serial.print("Password: ");
  Serial.println((char*) password);
  
  if (memcmp(password, LOCK_PASSWORD, strlen((char*)LOCK_PASSWORD)) == 0) {
    Serial.println("Password correct, unlocking!");
    door_locked = false;
  } else {
    Serial.print("password: ");
    Serial.println((char*)password);
    Serial.print("set password: ");
    Serial.println((char*)LOCK_PASSWORD);
    Serial.println("Incorrect password, door remains locked.");
  }
}

#endif // USE_EMBER

/******************************************
  Setup
******************************************/
void setup() {
  Serial.begin(115200);

  pinMode(PIR_PIN, INPUT);
  pinMode(UNLOCK_BUTTON_PIN, INPUT_PULLUP);

  #ifdef USE_EMBER

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);

  // Create the main container instance.
  container = std::make_unique<ember::Container>();

  #endif // USE_EMBER

  Serial.println("Door Lock System Initialized");
}

/******************************************
  Loop
******************************************/
void loop() {
  #ifdef USE_EMBER

  ember::Container::PollResult result = container->poll();
  if (result.should_stop) {
    Serial.println(
      (result.status == 0) ? "Finished executing." : "Container exited with an error!"
    );
    exit(result.status);
  }

  #else

  object_detected = digitalRead(PIR_PIN) == HIGH;
  
  // Auto-lock after 5 seconds if no object detected
  if (!door_locked && !object_detected && (millis() - unlocked_at) >= 5000) {
    Serial.println("Automatically locking door.");
    door_locked = true;
  }
  
  // Check unlock button
  if (digitalRead(UNLOCK_BUTTON_PIN) == LOW && door_locked) {
    Serial.println("Unlock button pressed.");
    unlock();
    unlocked_at = millis();
  }

  #endif // USE_EMBER
}