# gdal

[![Build Status](https://travis-ci.org/georust/gdal.png?branch=master)](https://travis-ci.org/georust/gdal)

[Documentation](https://georust.github.io/gdal)

[GDAL](http://gdal.org/) bindings for [Rust](http://www.rust-lang.org/).


### Installation

Comes with prebuild binaries for `GDAL` - to use, specify `features = ["bindgen"]`
in your `Cargo.toml`. See full readme [here](./gdal-sys/Readme.md)

### Features
So far, you can:

* open a raster dataset for reading/writing
* get size and number of bands
* get/set projection and geo-transform
* read and write raster data
* warp between datasets
* read and write vector data
* access metadata

Many raster and vector functions are not available. Patches welcome :)
