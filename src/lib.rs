#![crate_name = "gdal"]
#![crate_type = "lib"]
#![doc = include_str!("../README.md")]

//! ## Examples
//!
//! ### Raster
//!
//! This example shows opening a raster [`Dataset`] and using a few of the data access methods.
//! The GDAL [Raster Data Model](https://gdal.org/user/raster_data_model.html) document provides
//! details on the various constructs involved, as this example only touches the surface.
//!
//! ```rust, no_run
//! // `Dataset` is required for opening files. `Metadata` is required to enable reading of some
//! // general information properties, such as `description`.
//! use gdal::{Dataset, Metadata};
//! # fn main() -> gdal::errors::Result<()> {
//! // The `Dataset::open` function is used to open all datasets, regardless of type.
//! // There's a `Dataset:open_ex` variant which provides some additional options.
//! let dataset = Dataset::open("fixtures/tinymarble.tif")?;
//! // The `description` property for a `Dataset` is often (but not necessarily) the file name
//! println!("Dataset description: {}", dataset.description()?);
//! let band_count = dataset.raster_count();
//! println!("Number of bands: {band_count}");
//! // Beware! In GDAL, band indexes are 1-based!
//! for i in 1..=band_count {
//!     println!("  Band {i}");
//!     let band = dataset.rasterband(i)?;
//!     // Depending on the file, the description field may be the empty string :(
//!     println!("    Description: '{}'", band.description()?);
//!     // In GDAL, all no-data values are coerced to floating point types, regardless of the
//!     // underlying pixel type.
//!     println!("    No-data value: {:?}", band.no_data_value());
//!     println!("    Pixel data type: {}", band.band_type());
//!     // Scale and offset are often used with integral pixel types to convert between pixel value
//!     // to some physical unit (e.g. watts per square meter per steradian)
//!     println!("    Scale: {:?}", band.scale());
//!     println!("    Offset: {:?}", band.offset());
//!     // In GDAL you can read arbitrary regions of the raster, and have them up- or down-sampled
//!     // when the output buffer size is different from the read size. The terminology GDAL
//!     // uses takes getting used to. All parameters here are in pixel coordinates.
//!     // Also note, tuples are in `(x, y)`/`(cols, rows)` order.
//!     // `window` is the (x, y) coordinate of the upper left corner of the region to read.
//!     let window = (20, 30);
//!     // `window_size` is the amount to read `(cols, rows)`
//!     let window_size = (2, 3);
//!     // `size` is the output buffer size. If this is different from `window_size`, then
//!     // the `resample_alg` parameter below becomes relevant.
//!     let size = (2, 3);
//!     // Options here include `NearestNeighbor` (default), `Bilinear`, `Cubic`, etc.
//!     let resample_alg = None;
//!     // Note the `u8` type parameter. GDAL will convert the native pixel type to whatever is
//!     // specified here... which may or may not be right for your use case!
//!     let rv = band.read_as::<u8>(window, window_size, size, resample_alg)?;
//!     // `Rasterband::read_as` returns a `Buffer` struct, which contains the shape of the output
//!     // `(cols, rows)` and a `Vec<_>` containing the pixel values.
//!     println!("    Data size: {:?}", rv.size);
//!     println!("    Data values: {:?}", rv.data);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The resulting output is:
//!
//! ```text
//! Dataset description: fixtures/tinymarble.tif
//! Number of bands: 3
//!   Band 1
//!     Description: ''
//!     No-data value: None
//!     Pixel data type: 1
//!     Scale: None
//!     Offset: None
//!     Data size: (2, 3)
//!     Data values: [47, 74, 77, 118, 98, 122]
//!   Band 2
//!     Description: ''
//!     No-data value: None
//!     Pixel data type: 1
//!     Scale: None
//!     Offset: None
//!     Data size: (2, 3)
//!     Data values: [50, 79, 77, 118, 95, 119]
//!   Band 3
//!     Description: ''
//!     No-data value: None
//!     Pixel data type: 1
//!     Scale: None
//!     Offset: None
//!     Data size: (2, 3)
//!     Data values: [71, 94, 79, 115, 77, 98]
//! ```
//!
//!
//! ### Vector
//!
//! This example opens a vector [`Dataset`] and iterates over the various levels of structure within it.
//! The GDAL vector data model is quite sophisticated, so please refer to the GDAL
//! [Vector Data Model](https://gdal.org/user/vector_data_model.html) document for specifics.
//!
//! ```rust, no_run
//! use gdal::{Dataset, Metadata};
//! // The `LayerAccess` trait enables reading of vector specific fields from the `Dataset`.
//! use gdal::vector::LayerAccess;
//! # fn main() -> gdal::errors::Result<()> {
//! use gdal::errors::GdalError;
//! use gdal::vector::geometry_type_to_name;
//! let dataset = Dataset::open("fixtures/roads.geojson")?;
//! println!("Dataset description: {}", dataset.description()?);
//! let layer_count = dataset.layer_count();
//! println!("Number of layers: {layer_count}");
//! // Unlike raster bands, layers are zero-based
//! for l in 0..layer_count {
//!     // We have to get a mutable borrow on the layer because the `Layer::features` iterator
//!     // requires it.
//!     let mut layer = dataset.layer(l)?;
//!     let feature_count = layer.feature_count();
//!     println!("  Layer {l}, name='{}', features={}", layer.name(), feature_count);
//!     for feature in layer.features() {
//!         // The fid is important in cases where the vector dataset is large can you
//!         // need random access.
//!         let fid = feature.fid().unwrap_or(0);
//!         // Summarize the geometry
//!         let geometry = feature.geometry();
//!         let geom_type = geometry_type_to_name(geometry.geometry_type());
//!         let geom_len = geometry.get_point_vec().len();
//!         println!("    Feature fid={fid:?}, geometry_type='{geom_type}', geometry_len={geom_len}");
//!         // Get all the available fields and print their values
//!         for field in feature.fields() {
//!             let name = field.0;
//!             let value = field.1.and_then(|f| f.into_string()).unwrap_or("".into());
//!             println!("      {name}={value}");
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The resulting (truncated) output looks like this:
//!
//! ```text
//! Dataset description: fixtures/roads.geojson
//! Number of layers: 1
//!   Layer 0, name='roads', features=21
//!     Feature fid=236194095, geometry_type='Line String', geometry_len=3
//!       kind=path
//!       sort_key=
//!       is_link=no
//!       is_tunnel=no
//!       is_bridge=no
//!       railway=
//!       highway=footway
//!     Feature fid=236194098, geometry_type='Line String', geometry_len=3
//!       kind=path
//!       sort_key=
//!       is_link=no
//!       is_tunnel=no
//!       is_bridge=no
//!       railway=
//!       highway=footway
//!     Feature fid=236194101, geometry_type='Line String', geometry_len=4
//!       kind=path
//!       sort_key=
//!       is_link=no
//!       is_tunnel=no
//!       is_bridge=no
//!       railway=
//!       highway=footway
//! ...
//! ```

pub use version::version_info;

pub mod config;
pub mod cpl;
mod dataset;
mod driver;
pub mod errors;
mod gdal_major_object;
mod metadata;
pub mod programs;
pub mod raster;
pub mod spatial_ref;
#[cfg(test)]
pub mod test_utils;
mod utils;
pub mod vector;
pub mod version;
pub mod vsi;

pub use dataset::{
    Dataset, DatasetOptions, GdalOpenFlags, GeoTransform, GeoTransformEx, LayerIterator,
    LayerOptions, Transaction,
};
pub use driver::Driver;
pub use metadata::Metadata;

#[cfg(test)]
fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.00001);
    assert!(f > 0.99999);
}
