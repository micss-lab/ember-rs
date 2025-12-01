#define USE_EMBER // Uncomment this to use the ember library.

#include "common.h"

#define PIR_PIN 18
#define UNLOCK_BUTTON_PIN 22

const char LOCK_PASSWORD[5] = "1234";

#ifdef USE_EMBER
#include "Ember.h"

#include "agents/LockAgent.h"
#include "agents/PirAgent.h"
#endif // USE_EMBER


#ifdef USE_EMBER

std::unique_ptr<ember::Container> container;

#else 

bool door_locked = true;
unsigned long unlocked_at = 0;
bool object_detected = false;


#endif // USE_EMBER

/******************************************
  Helper Functions
******************************************/
#ifndef USE_EMBER


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

unsigned long current_time = micros();
unsigned int ticks = 0;

/******************************************
  Setup
******************************************/
void setup() {
  const unsigned long start_micros = micros();

  Serial.begin(115200);

  pinMode(PIR_PIN, INPUT);
  pinMode(UNLOCK_BUTTON_PIN, INPUT_PULLUP);

  Serial.print("Peripheral setup: ");
  Serial.println(micros() - start_micros);

  #ifdef USE_EMBER

  const unsigned long ember_setup_micros = micros();

  // Initialize the embers required resources.
  ember::initialize(ember::logging::LogLevel::Debug);

  // Create the main container instance.
  container = std::make_unique<ember::Container>();

  auto lock_agent = agents::lock::create_lock_agent(UNLOCK_BUTTON_PIN);
  auto pir_agent = agents::pir::create_pir_agent(PIR_PIN);

  container->add_agent(std::move(lock_agent));
  container->add_agent(std::move(pir_agent));

  Serial.print("Ember setup: ");
  Serial.println(micros() - ember_setup_micros);

  #endif // USE_EMBER

  Serial.println("Door Lock System Initialized");
}

/******************************************
  Loop
******************************************/
void loop() {
  ticks += 1;
  const unsigned long new_time = micros();
  if (static_cast<float>(new_time - current_time) / 1e6 >= 1.0) {
    Serial.print("tps: ");
    Serial.println(ticks);

    ticks = 0;
    current_time = new_time;
  }

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