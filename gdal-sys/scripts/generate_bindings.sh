#!/bin/bash
set -e

# install dependencies
apt update && apt install -y libclang-dev mingw-w64 gcc-i686-linux-gnu pkg-config rustfmt xz-utils curl

# install bindgen using the prebuilt binary installer
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rust-lang/rust-bindgen/releases/download/v0.71.1/bindgen-cli-installer.sh | sh
export PATH="$HOME/.cargo/bin:$PATH"

# detect mingw gcc version (varies by ubuntu version: gcc-9 on 20.04, gcc-10 on 22.04, gcc-13 on 24.04)
MINGW64_GCC_DIR=$(ls -d /usr/lib/gcc/x86_64-w64-mingw32/*-win32 2>/dev/null | head -1)
MINGW32_GCC_DIR=$(ls -d /usr/lib/gcc/i686-w64-mingw32/*-win32 2>/dev/null | head -1)
echo "Detected mingw64 gcc: $MINGW64_GCC_DIR"
echo "Detected mingw32 gcc: $MINGW32_GCC_DIR"

# create output directory if needed
mkdir -p /gdal-sys/prebuilt-bindings/$GDAL_VERSION

# 64 bit linux/macos
echo "Generating 64-bit Linux bindings..."
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_x86_64-unknown-linux-gnu.rs

# 32 bit linux/macos
echo "Generating 32-bit Linux bindings..."
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h -- -target i686-unknown-linux-gnu --sysroot /usr/i686-linux-gnu/ -I /usr/include > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_i686-unknown-linux-gnu.rs

# remove conflicting system headers before generating windows bindings
rm -f /usr/include/stdio.h /usr/include/stdlib.h /usr/include/limits.h /usr/include/features-time64.h /usr/include/features.h /usr/include/malloc.h /usr/include/string.h /usr/include/ctype.h /usr/include/errno.h /usr/include/math.h /usr/include/stdint.h /usr/include/time.h

# 64 bit windows
# -D__CLANG_MAX_ALIGN_T_DEFINED prevents clang's max_align_t from conflicting with mingw's
echo "Generating 64-bit Windows bindings..."
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h -- -target x86_64-pc-windows-gnu -D__CLANG_MAX_ALIGN_T_DEFINED -I /usr/include/ -I $MINGW64_GCC_DIR/include > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_x86_64-pc-windows-gnu.rs

# 32 bit windows
echo "Generating 32-bit Windows bindings..."
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h -- -target i686-pc-windows-gnu -D__CLANG_MAX_ALIGN_T_DEFINED -I /usr/include/ -I $MINGW32_GCC_DIR/include/ > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_i686-pc-windows-gnu.rs

echo "Done generating all bindings!"
