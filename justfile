default:
    just --list

build-esp-bindings:
    cargo build --release --target riscv32imc-unknown-none-elf --package no-std-framework-bindings

update-arduino-lib: build-esp-bindings
    cp ./target/riscv32imc-unknown-none-elf/release/libframework_core.a ./arduino/libraries/Framework/src/esp32c3/FrameworkCore.a
    cp ./target/bindings.hpp ./arduino/libraries/Framework/src/FrameworkCore.h
