#include <FrameworkCore.h>

void setup() {
  Serial.begin(115200);
  Serial.println("Hello, ESP32-C3!");

  // Set log level to trace.
  ffi::initialize_logging(5);
  // Set up the global allocator of the library.
  ffi::initialize_allocator();

  // Create the main container instance.
  ffi::Container* container = ffi::container_new();
  Serial.println("Do we arrrive here?");
  // Start the container and check for errors.
  int failed = (bool) ffi::container_start(container);
  if (failed) {
    Serial.println("Container exited with an error!");
  }

  // Cleanup.
  ffi::container_free(container);

  Serial.println("Finished executing.");
}

void loop() {

}