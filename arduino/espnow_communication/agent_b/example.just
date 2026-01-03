default:
    @just --unstable --justfile ./example.just --list

build:
    arduino-cli compile --build-path ./build --fqbn esp32:esp32:esp32 --libraries ../../libraries --verbose

run: build
    espflash flash ./build/*.ino.elf --monitor
