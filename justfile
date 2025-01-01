mod bindings
import 'core/justfile'
import 'tests/justfile'

default:
    @just --unstable --list

check:
    cd core && cargo check
    cd bindings && cargo check-esp
    cd tests && cargo check-local --tests
    cd examples && cargo check
