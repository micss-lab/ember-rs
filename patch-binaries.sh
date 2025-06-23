#!/usr/bin/env bash
set -euo pipefail

# Helper: get Nix store path for a package
get_pkg () {
    nix --extra-experimental-features nix-command --extra-experimental-features flakes eval -f '<nixpkgs>' --raw "$1"
}

# Packages needed for RPATH
GLIBC=$(get_pkg 'glibc')
GCC_LIB=$(get_pkg 'stdenv.cc.cc.lib')
ZLIB=$(get_pkg 'zlib')
XZ=$(get_pkg 'xz')
BZIP2=$(get_pkg 'bzip2')
LIBSSH2=$(get_pkg 'libssh2')
OPENSSL=$(get_pkg 'openssl')
CURL=$(get_pkg 'curl')
LLVM_LIB=$(get_pkg 'llvmPackages.libllvm')
LIBEDIT=$(get_pkg 'libedit')
NCURSES=$(get_pkg 'ncurses')
LIBXML2=$(get_pkg 'libxml2')
SQLITE=$(get_pkg 'sqlite')
PKGCONFIG=$(get_pkg 'pkg-config')

# Compose the RPATH as a colon-separated list
ALL_LIBS="${GLIBC}/lib:${GCC_LIB}/lib:${ZLIB}/lib:${XZ}/lib:${BZIP2}/lib:${LIBSSH2}/lib:${OPENSSL}/lib:${CURL}/lib:${LLVM_LIB}/lib:${LIBEDIT}/lib:${NCURSES}/lib:${LIBXML2}/lib:${SQLITE}/lib"

# Find the CORRECT dynamic linker for the architecture, directly from glibc
DYNAMIC_LINKER_PATH="$(find "$GLIBC/lib" -type f -name 'ld-linux-*.so.2' | head -n1)"
if [[ ! -x "$DYNAMIC_LINKER_PATH" ]]; then
    echo "ERROR: Could not find dynamic linker in $GLIBC/lib"
    exit 1
fi

# Patch function, safe ELF-detection and no symlinks/scripts
patch_elf() {
    file="$1"
    # Only regular files, non-symlinks, and ELF binaries
    if [[ -f "$file" && ! -L "$file" ]] && file "$file" | grep -qE 'ELF (64|32)-bit .* (executable|shared object)'; then
        echo "  Patching $file"
        patchelf --set-interpreter "$DYNAMIC_LINKER_PATH" \
                 --set-rpath "${HOME}/.rustup/toolchains/${toolchain}/lib:${ALL_LIBS}" \
                 "$file" 2> /dev/null || true
    fi
}

# Patch all ELF binaries and shared objects in toolchain(s)
for toolchain in esp; do
    echo "Patching toolchain: $toolchain"
    find "$HOME/.rustup/toolchains/$toolchain" -type f -o -type l | while read -r file; do
        patch_elf "$file"
    done
    echo "Done patching $toolchain."
done
