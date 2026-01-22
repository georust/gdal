# Updating bundled gdal version for gdal-src

Perform the following steps:

```
git submodule init
git submodule update
cd gdal-src/source
git pull
git checkout v3.8.4 # corresponds to the tag you want to update to
cd ../../
git add gdal-src/source
git commit -m "Update bundled gdal version to 3.8.4"
```

These steps assume that there are no fundamental changes to the gdal build system.

# Generating Bindings

## Using the Scripts (Recommended)

Scripts are provided in `gdal-sys/scripts/` to automate binding generation:

```bash
# Generate bindings for all GDAL versions (3.4 through 3.12)
cd gdal-sys/scripts
./generate_all_bindings.sh

# Generate bindings for a specific GDAL version
./generate_all_bindings.sh 3_11
```

The scripts use Docker images for each GDAL version and handle differences in Ubuntu base versions (mingw gcc versions vary: 9 on Ubuntu 20.04, 10 on 22.04, 13 on 24.04).

## Manual Generation

If you prefer to generate bindings manually or need to debug the process:

```bash
docker run -it --rm -v ./gdal-sys:/gdal-sys:z -w /gdal-sys -e GDAL_VERSION=3_12 ghcr.io/osgeo/gdal:ubuntu-full-3.12.0 bash
# everything from now on is inside of the container

# install mingw toolchain for generating windows bindings
# install libclang for bindgen
# gcc-i686-linux-gnu to generate bindings for 32 bit linux
apt update && apt install -y libclang-dev mingw-w64 gcc-i686-linux-gnu pkg-config rustfmt xz-utils

# install bindgen
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rust-lang/rust-bindgen/releases/download/v0.71.1/bindgen-cli-installer.sh | sh
source $HOME/.cargo/env

# create the output directory for the prebuild bindings if necessary
mkdir /gdal-sys/prebuilt-bindings/$GDAL_VERSION

# if you update these commands consider updating the command in gdal-sys/build.rs
# make sure to use the same bindgen flags (everything before wrapper) for
# all targets
#
# 64 bit linux/macos
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_x86_64-unknown-linux-gnu.rs
# 32 bit linux/macos
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h -- -target i686-unknown-linux-gnu --sysroot /usr/i686-linux-gnu/ -I /usr/include > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_i686-unknown-linux-gnu.rs

# make sure we don't get the wrong system headers
rm /usr/include/stdio.h /usr/include/stdlib.h /usr/include/limits.h /usr/include/features-time64.h /usr/include/features.h /usr/include/malloc.h /usr/include/string.h /usr/include/ctype.h /usr/include/errno.h /usr/include/math.h /usr/include/stdint.h /usr/include/time.h

# 64 bit windows
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h -- -target x86_64-pc-windows-gnu -I /usr/include/ -I /usr/lib/gcc/x86_64-w64-mingw32/13-win32/include > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_x86_64-pc-windows-gnu.rs
# 32 bit windows
bindgen --rust-target 1.77 --rust-edition 2021 --constified-enum-module ".*" --ctypes-prefix ::std::ffi --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI|VRT).*" /gdal-sys/wrapper.h -- -target i686-pc-windows-gnu -I /usr/include/ -I /usr/lib/gcc/i686-w64-mingw32/13-win32/include/ > /gdal-sys/prebuilt-bindings/$GDAL_VERSION/gdal_i686-pc-windows-gnu.rs
```
