# Changes

## Unreleased

- **Breaking**: Implement `TryFrom` instead of `From` to convert from gdal geometries to `geo-types`. This avoids a possible panic on unsupported geometries and returns an error instead.
- Add wrapper for `OGR_G_Buffer`.
- Add support for raster dataset creation options. A new struct (`RasterCreationOption`) and function (`driver.create_with_band_type_with_options()`) are now available for this.

  - <https://github.com/georust/gdal/pull/193>

```rust
let driver = Driver::get("GTiff").unwrap();
let options = &[
    RasterCreationOption {
        key: "COMPRESS",
        value: "LZW",
    },
    RasterCreationOption {
        key: "TILED",
        value: "YES",
    },
];
let mut dataset = driver
    .create_with_band_type_with_options::<u8>("testing.tif", 2048, 2048, 1, options)
    .unwrap();
```

- **Breaking**: Add support to select a resampling algorithm when reading a raster

  - <https://github.com/georust/gdal/pull/141>

  Now, it is necessary to provide a `Option<ResampleAlg>` when reading a raster.
  If `None`, it uses `ResampleAlg::NearestNeighbour` which was the
  default behavior.

- **Breaking**: Make `Layer::features` iterator reset to
  beginning, and borrow mutably.

  - closes <https://github.com/georust/gdal/issues/159>

- **Breaking**: [Enforce borrow
  semantics](https://github.com/georust/gdal/pull/161) on
  methods of `Dataset`, `RasterBand`, and `Layer`.

  1. Methods that do not modify the underlying structure take `&self`.
  1. Methods that modify the underlying structure take `&mut self`.

  ```rust
  let ds = Dataset::open(...);

  // ds need not be mutable to open layer
  let mut band = ds.rasterband(1)?;

  // band needs to be mutable to set no-data value
  band.set_no_data_value(0.0)?;
  ```

- **Breaking**: Upgrade to `ndarray 0.15`
  - <https://github.com/georust/gdal/pull/175>
- Implement wrapper for `OGR_L_TestCapability`

  - <https://github.com/georust/gdal/pull/160>

- **Breaking**: Use `DatasetOptions` to pass as `Dataset::open_ex` parameters and
  add support for extended open flags.

  ```rust
      use gdal::{ Dataset, DatasetOptions }

      let dataset = Dataset::open_ex(
          "roads.geojson",
          DatasetOptions {
              open_flags: GdalOpenFlags::GDAL_OF_UPDATE|GdalOpenFlags::GDAL_OF_VECTOR,
              ..DatasetOptions::default()
          }
      )
      .unwrap();
  ```

  `GDALAccess` values are supported using [`From`] implementation

  ```rust
      Dataset::open_ex(
          "roads.geojson",
          DatasetOptions {
              open_flags: GDALAccess::GA_Update.into(),
              ..DatasetOptions::default()
          },
      )
      .unwrap();
  ```

- Add more functions to SpatialRef implementation
  - <https://github.com/georust/gdal/pull/145>
- **Breaking**: Change `Feature::field` return type from
  `Result<FieldValue>` to `Result<Option<FieldValue>>`. Fields
  can be null. Before this change, if a field was null, the value
  returned was the default value for the underlying type.
  However, this made it impossible to distinguish between null
  fields and legitimate values which happen to be default value,
  for example, an Integer field that is absent (null) from a 0,
  which can be a valid value. After this change, if a field is
  null, `None` is returned, rather than the default value.

  If you happened to rely on this behavior, you can fix your code
  by explicitly choosing a default value when the field is null.
  For example, if you had this before:

  ```rust
  let str_var = feature.field("string_field")?
      .into_string()
      .unwrap();
  ```

  You could maintain the old behavior with:

  ```rust
  use gdal::vector::FieldValue;

  let str_var = feature.field("string_field")?
      .unwrap_or(FieldValue::StringValue("".into()))
      .into_string()
      .unwrap();
  ```

  - <https://github.com/georust/gdal/pull/134>

- Fixed potential race condition wrt. GDAL driver initialization
  - <https://github.com/georust/gdal/pull/166>
- Add basic support to read overviews
- Added a `Dataset::build_overviews` method
  - <https://github.com/georust/gdal/pull/164>
- BREAKING: update geo-types to 0.7.0. geo-types Coordinate<T> now implement `Debug`
  - <https://github.com/georust/gdal/pull/146>
- Deprecated `SpatialRef::get_axis_mapping_strategy` - migrate to
  `SpatialRef::axis_mapping_strategy` instead.
- Add support for reading and setting rasterband colour interpretations
  - <https://github.com/georust/gdal/pull/144>
- Add `Geometry::from_wkb` and `Geometry::wkb` functions to convert from/to
  Well-Known Binary
  - <https://github.com/georust/gdal/pull/173>
- Fixed memory leak in `Geometry::from_wkt`
  - <https://github.com/georust/gdal/pull/172>

- **Breaking**: Changed `Dataset::create_layer` to take a new `LayerOptions`
  struct instead of separate arguments.

  Before:

  ```rust
  ds.create_layer("roads", None, wkbLineString)
  ```

  After (all fields have usable default values):

  ```rust
  use gdal::LayerOptions;
  ds.create_layer(LayerOptions {
    name: "roads",
    ty: wkbLineString,
    ..Default::default()
  });
  ```

  This change also removed `Dataset::create_layer_blank()`. Use
  `Dataset::create_layer(Default::default())` instead.

  - <https://github.com/georust/gdal/pull/186>

## 0.7.1

- fix docs.rs build for gdal-sys
  - <https://github.com/georust/gdal/pull/128>

## 0.6.0 - 0.7.0

- Dataset layer iteration and FieldValue types
  - https://github.com/georust/gdal/pull/126
- Fix i8 ptr instead of c_char ptr passed to OSRImportFromESRI()
  - <https://github.com/georust/gdal/pull/123>
- Rename spatial_reference to spatial_ref
  - <https://github.com/georust/gdal/pull/114>
- Replace get_extent force flag by get_extent and try_get_extent
  - <https://github.com/georust/gdal/pull/113>
- Add support for transactions on datasets
  - <https://github.com/georust/gdal/pull/109>
- Add feature_count{,\_force} and implement Iterator::size_hint
  - <https://github.com/georust/gdal/pull/108>
- Replace failure with thiserror
  - <https://github.com/georust/gdal/pull/103>
- Ability to read into preallocated slice for rasterband
  - <https://github.com/georust/gdal/pull/100>
- Datasets are Send (requires GDAL >= 2.3)
  - <https://github.com/georust/gdal/pull/99>
- User GDALOpenEx
  - <https://github.com/georust/gdal/pull/97>
- GDAL 2.0 conform structure / drop GDAL 1.x
  - <https://github.com/georust/gdal/pull/96>
- Inplace functions use mutable refs
  - <https://github.com/georust/gdal/pull/93>
- Detect GDAL version at build time / remove version features
  - <https://github.com/georust/gdal/pull/92>
- Add support for delaunay_triangulation and simplify functions
  - <https://github.com/georust/gdal/pull/91>
- Add support for 3d points
  - <https://github.com/georust/gdal/pull/90>
- Additional metadata retrieval options
  - <https://github.com/georust/gdal/pull/88>
- Support for GDAL 3 in CI
  - <https://github.com/georust/gdal/pull/86>
- Support for Integer64
  - <https://github.com/georust/gdal/pull/80>
- Geometry Intersection trait
  - <https://github.com/georust/gdal/pull/78>
- Rust 2018
  - <https://github.com/georust/gdal/pull/75>
- support for date and time fields
  - <https://github.com/georust/gdal/pull/72>
- Prebuild bindings
  - <https://github.com/georust/gdal/pull/69>
- Support for ndarray
  - <https://github.com/georust/gdal/pull/68>

## 0.5.0

- [Bump geo-types from 0.3 -> 0.4](https://github.com/georust/gdal/pull/71)
- [Allow reading block-size of Rasters](https://github.com/georust/gdal/pull/67)
- [Add prebuilt-bindings GDAL 2.3 and GDAL 2.4](https://github.com/georust/gdal/pull/69)
- [Make GdalType trait public](https://github.com/georust/gdal/pull/66)
- [RasterBand to Ndarray, with failure](https://github.com/georust/gdal/pull/68)

## 0.4.0

- [Migrate to the `geo-types` crate](https://github.com/georust/gdal/pull/60)
- [Replace `error-chain` with `failure`](https://github.com/georust/gdal/pull/58)
- [Use `bindgen` to generate the low-level bindings](https://github.com/georust/gdal/pull/55)

## 0.3.0

- [Add support for creating a SpatialRef from a esri "wkt" definition](https://github.com/georust/gdal/pull/37)
- [Travis now uses GDAL 2.x](https://github.com/georust/gdal/pull/36)
- [API extensions](https://github.com/georust/gdal/pull/35)
- [Extend the existing possibilities of writing ogr datasets](https://github.com/georust/gdal/pull/31)
- [Allow to transform ogr geometries to other SRS](https://github.com/georust/gdal/pull/29)
- [Move ffi into a seperate crate](https://github.com/georust/gdal/pull/26)
- [Added rasterband.rs and moved all band functions](https://github.com/georust/gdal/pull/24)

## 0.2.1

- [First version of metadata handling](https://github.com/georust/gdal/pull/21)
