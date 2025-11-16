#!/usr/bin/env bash

# Recently, running a project using the C++ Ember library crashes due to a
# failure in static initialization. Even before any library, or user-provided
# code is run.
#
# After some debugging, it turns out that compiling the `bindings`, rust
# includes weak symbols in the library such as memcpy, memmove, etc.
# These symbols are (for some unknown reason), not being overridden anymore
# when linking the Arduino sketch.
#
# When the Esp is setting up the call stack, it makes use of one of these
# functions. As this function now originates from rust, it is possible that
# this code is not yet in accessible memory regions. Thus a fatal crash occurs.
#
# This script removes any object files from the static library that contain
# these functions or any other compiler builtin functions. This solves the
# issue for now, but to me, is not a valid "fix".
#
# NOTE: This script is mostly A.I-generated.

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <library.a>"
    exit 1
fi

LIB_NAME=$(basename "$1")
LIB_DIR_PATH="$(cd $(dirname "$1") && pwd -P)"
LIB_PATH="$LIB_DIR_PATH/$LIB_NAME"
TMP_DIR=$(mktemp -d)

echo "Working in temporary directory: $TMP_DIR"

# Extract to tmp directory
cd "$TMP_DIR"
ar x "$LIB_PATH"

# Find and list all objects with compiler_builtins and mem functions.
echo "Objects containing compiler_builtins mem functions:"
for obj in *.o; do
    if xtensa-esp32-elf-nm "$obj" 2>/dev/null | grep -q "_ZN17compiler_builtins3mem"; then
        echo "  $obj"
        rm -f "$obj"
    fi
done

# Recreate the archive
ar rcs "$LIB_NAME" *.o

# Move back to original location
mv "$LIB_NAME" "$LIB_PATH"

# Verify and fail if mem symbols still exist
echo "Remaining mem symbols:"
if xtensa-esp32-elf-nm -g "$LIB_PATH" | grep -E "memcpy|memmove|memset" | grep -v " U "; then
    echo "ERROR: mem symbols still found in library after removal!"
    # Cleanup
    cd - > /dev/null
    rm -rf "$TMP_DIR"
    exit 1
else
    echo "  (none found - success)"
fi

# Cleanup
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "Done. Library updated: $LIB_PATH"
