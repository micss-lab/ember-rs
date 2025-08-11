CARGO_EXTRA_ARGS := "--locked"

mod bindings
mod arduino

default:
    @just --unstable --list
