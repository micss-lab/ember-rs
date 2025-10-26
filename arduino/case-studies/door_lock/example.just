SERIAL_PORT := `ls /dev/ttyACM* /dev/ttyUSB* 2>/dev/null | head -n 1`

default:
    @just --unstable --justfile ./example.just --list

build:
    arduino-cli compile --build-path ./build --fqbn esp32:esp32:esp32 --libraries ../../libraries --verbose

run: build
    arduino-cli upload --fqbn esp32:esp32:esp32 --input-dir ./build --port {{SERIAL_PORT}} --verbose
    espflash monitor
