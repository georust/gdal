# gdal-sys

[![Build Status](https://travis-ci.org/georust/gdal.png?branch=master)](https://travis-ci.org/georust/gdal)

Low level [GDAL](http://gdal.org/) bindings for [Rust](http://www.rust-lang.org/).
The build script will try to auto-dectect the installed GDAL version.

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

## Generated bindings

By default, gdal-sys will detect the version of libgdal you have installed and
attempt to use prebuilt bindings corresponding to that version. Alternatively,
you can generate your own bindings from your libgdal installation by specifying
the `bindgen` feature.

