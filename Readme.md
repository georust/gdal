## Rust-GDAL ##

[![Build Status](https://travis-ci.org/mgax/rust-gdal.png?branch=master)](https://travis-ci.org/mgax/rust-gdal)

[GDAL](http://gdal.org/) bindings for [Rust](http://www.rust-lang.org/).
The library tracks rust master which is rapidly evolving.

So far, you can:

* open a raster dataset for reading/writing
* get size and number of bands
* get/set projection and geo-transform
* read and write raster data
* warp between datasets
* read vector data
* convert between [PROJ.4](http://trac.osgeo.org/proj/) projections

Many raster and vector functions are not available. Patches welcome :)
