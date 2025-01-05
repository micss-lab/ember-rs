CARGO_EXTRA_ARGS := "--locked"

mod bindings
mod examples
mod tests
import 'core/justfile'

default:
    @just --unstable --list

check:
    cd core && cargo check {{CARGO_EXTRA_ARGS}}
    cd bindings && cargo check-esp {{CARGO_EXTRA_ARGS}}
    cd tests && cargo check-local --tests {{CARGO_EXTRA_ARGS}}
    cd examples && cargo check-local {{CARGO_EXTRA_ARGS}}
    cd examples && cargo check-esp {{CARGO_EXTRA_ARGS}}
