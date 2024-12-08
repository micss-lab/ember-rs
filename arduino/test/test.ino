#include <FrameworkCore.h>

void setup() {
  // put your setup code here, to run once:
  Serial.begin(115200);
  Serial.println("Hello, ESP32-C3!");

  ffi::initialize_logging(5);
  ffi::Container* container = ffi::container_new();
}

void loop() {
  // put your main code here, to run repeatedly:
  delay(10); // this speeds up the simulation
}
