# gdal-sys

[![Build Status](https://travis-ci.org/georust/rust-gdal.png?branch=master)](https://travis-ci.org/georust/rust-gdal)

Low level [GDAL](http://gdal.org/) bindings for [Rust](http://www.rust-lang.org/).

Contains:

* mapping of data types
* raster (GDAL) and vector (ORG) operations
* error handling
* spatial reference operations

## build

The build script should work an Linux and Windows systems.

On Windows the `GDAL_HOME` environment variable is expected to point to the `GDAL` folder.

* windows-msvc requires `gdal_i.lib` to be found in `GDAL_HOME\lib`.
* windows-gnu requires either `gdal_i.lib` in `GDAL_HOME\lib` OR `gdal{version}.dll` in `GDAL_HOME\bin`.