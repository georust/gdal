# Changes
## Unreleased
* Add basic support to read overviews
* BREAKING: update geo-types to 0.7.0. geo-types Coordinate<T> now implement `Debug`
  * <https://github.com/georust/gdal/pull/146>

## 0.7.1
* fix docs.rs build for gdal-sys
    * <https://github.com/georust/gdal/pull/128>

## 0.6.0 - 0.7.0
* Dataset layer iteration and FieldValue types
    * https://github.com/georust/gdal/pull/126
* Fix i8 ptr instead of c_char ptr passed to OSRImportFromESRI()
    * <https://github.com/georust/gdal/pull/123>
* Rename spatial_reference to spatial_ref
    * <https://github.com/georust/gdal/pull/114>
* Replace get_extent force flag by get_extent and try_get_extent
    * <https://github.com/georust/gdal/pull/113>
* Add support for transactions on datasets
    * <https://github.com/georust/gdal/pull/109>
* Add feature_count{,_force} and implement Iterator::size_hint
    * <https://github.com/georust/gdal/pull/108>
* Replace failure with thiserror
    * <https://github.com/georust/gdal/pull/103>
* Ability to read into preallocated slice for rasterband
    * <https://github.com/georust/gdal/pull/100>
* Datasets are Send (requires GDAL >= 2.3)
    * <https://github.com/georust/gdal/pull/99>
* User GDALOpenEx
    * <https://github.com/georust/gdal/pull/97>
* GDAL 2.0 conform structure / drop GDAL 1.x
    * <https://github.com/georust/gdal/pull/96>
* Inplace functions use mutable refs
    * <https://github.com/georust/gdal/pull/93>
* Detect GDAL version at build time / remove version features
    * <https://github.com/georust/gdal/pull/92>
* Add support for delaunay_triangulation and simplify functions
    * <https://github.com/georust/gdal/pull/91>
* Add support for 3d points
    * <https://github.com/georust/gdal/pull/90>
* Additional metadata retrieval options
    * <https://github.com/georust/gdal/pull/88>
* Support for GDAL 3  in CI
    * <https://github.com/georust/gdal/pull/86>
* Support for Integer64
    * <https://github.com/georust/gdal/pull/80>
* Geometry Intersection trait
    * <https://github.com/georust/gdal/pull/78>
* Rust 2018
    * <https://github.com/georust/gdal/pull/75>
* support for date and time fields
    * <https://github.com/georust/gdal/pull/72>
* Prebuild bindings
    * <https://github.com/georust/gdal/pull/69>
* Support for ndarray
    * <https://github.com/georust/gdal/pull/68>

## 0.5.0

* [Bump geo-types from 0.3 -> 0.4](https://github.com/georust/gdal/pull/71)
* [Allow reading block-size of Rasters](https://github.com/georust/gdal/pull/67)
* [Add prebuilt-bindings GDAL 2.3 and GDAL 2.4](https://github.com/georust/gdal/pull/69)
* [Make GdalType trait public](https://github.com/georust/gdal/pull/66)
* [RasterBand to Ndarray, with failure](https://github.com/georust/gdal/pull/68)

## 0.4.0
* [Migrate to the `geo-types` crate](https://github.com/georust/gdal/pull/60)
* [Replace `error-chain` with `failure`](https://github.com/georust/gdal/pull/58)
* [Use `bindgen` to generate the low-level bindings](https://github.com/georust/gdal/pull/55)

## 0.3.0

* [Add support for creating a SpatialRef from a esri "wkt" definition](https://github.com/georust/gdal/pull/37)
* [Travis now uses GDAL 2.x](https://github.com/georust/gdal/pull/36)
* [API extensions](https://github.com/georust/gdal/pull/35)
* [Extend the existing possibilities of writing ogr datasets](https://github.com/georust/gdal/pull/31)
* [Allow to transform ogr geometries to other SRS](https://github.com/georust/gdal/pull/29)
* [Move ffi into a seperate crate](https://github.com/georust/gdal/pull/26)
* [Added rasterband.rs and moved all band functions](https://github.com/georust/gdal/pull/24)

## 0.2.1

* [First version of metadata handling](https://github.com/georust/gdal/pull/21)
