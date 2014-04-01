## Rust-GDAL ##

[GDAL](http://gdal.org/) bindings for [Rust](http://www.rust-lang.org/).
The library tracks rust master which is rapidly evolving.

So far, you can:

* open a raster dataset for reading/writing
* get size and number of bands
* get/set projection and geo-transform
* read and write raster data
* warp between datasets
* convert between [PROJ.4](http://trac.osgeo.org/proj/) projections

There is no support for vector data and many raster functions are not
available. Patches welcome :)
