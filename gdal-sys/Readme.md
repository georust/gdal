# gdal-sys

[![Build Status](https://travis-ci.org/georust/gdal.png?branch=master)](https://travis-ci.org/georust/gdal)

Low level [GDAL](http://gdal.org/) bindings for [Rust](http://www.rust-lang.org/).

Contains:

* mapping of data types
* raster (GDAL) and vector (OGR) operations
* error handling
* spatial reference operations

## Build

The build script should work an Linux and Windows systems. It can be configured with a couple of environment variables:

* if `GDAL_INCLUDE_DIR` or `GDAL_LIB_DIR` are defined, they will be used
* otherwise, if `GDAL_HOME` is defined, the build script looks for `GDAL_HOME/include`, `GDAL_HOME/lib` and `GDAL_HOME/bin`
* finally, `pkg-config` is queried to determine the `GDAL` location
* you can define `GDAL_STATIC` to link `GDAL` statically

The include directories are only used if you choose to generate the bindings at build time.

On Linux, building should work out-of-the-box.

On Windows, the easiest solution is to point the `GDAL_HOME` environment variable to the `GDAL` folder.

* `windows-msvc` requires `gdal_i.lib` to be found in `%GDAL_HOME%\lib`.
* `windows-gnu` requires either `gdal_i.lib` in `%GDAL_HOME%\lib` OR `gdal{version}.dll` in `%GDAL_HOME%\bin`.

## Pre-generated bindings

By default, the bundled bindings will be used. To generate them when building the crate, the `bindgen` feature must be enabled.

You can enable one of the `min_gdal_version_X_Y` features to pick a specific version of the pre-generated bindings. For more information, see the [Cargo.toml](Cargo.toml) file.
