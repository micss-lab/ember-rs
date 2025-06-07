set -euo pipefail

# Helper to get Nix store paths for packages
get_pkg () {
    nix --extra-experimental-features nix-command --extra-experimental-features flakes eval -f '<nixpkgs>' --raw "$1"
}

# Collect all runtime library store paths (as in rust-overlay)
GLIBC=$(get_pkg 'stdenv.cc.libc')
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

# Find the path to the dynamic linker (must match the architecture of your binaries)
NIX_CC=${NIX_CC:-$(nix-shell -p hello --run 'echo $NIX_CC')}
DYNAMIC_LINKER="$(cat $NIX_CC/nix-support/dynamic-linker)"

# Patch function, modeled after rust-overlay's patching
patch_elf() {
    file="$1"
    # Only patch ELF executables or shared libraries
    if file "$file" | grep -qE 'ELF (64|32)-bit .* (executable|shared object)'; then
        patchelf --set-interpreter "$DYNAMIC_LINKER" \
                 --set-rpath "${HOME}/.rustup/toolchains/${toolchain}/lib:${ALL_LIBS}" \
                 "$file" || true
    fi
}

# Patch all ELF binaries and libraries in your toolchain(s)
for toolchain in esp; do
    echo "Patching $toolchain"
    find "$HOME/.rustup/toolchains/$toolchain" -type f | while read -r file; do
        patch_elf "$file"
    done
done

