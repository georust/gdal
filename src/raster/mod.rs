//! GDAL Raster Data API
//!
//! ## Example
//!
//! This example shows opening a raster [`Dataset`](crate::Dataset) and using a few of the data access methods.
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
//!     println!("    Data size: {:?}", rv.shape());
//!     println!("    Data values: {:?}", rv.data());
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
//!     ...
//! ```

pub use buffer::{Buffer, ByteBuffer};
#[cfg(all(major_ge_3, minor_ge_1))]
pub use mdarray::{
    Attribute, Dimension, ExtendedDataType, ExtendedDataTypeClass, Group, MDArray, MdStatisticsAll,
};
pub use rasterband::{
    CmykEntry, ColorEntry, ColorInterpretation, ColorTable, GrayEntry, Histogram, HlsEntry,
    PaletteInterpretation, RasterBand, ResampleAlg, RgbaEntry, StatisticsAll, StatisticsMinMax,
};
pub use rasterize::{rasterize, BurnSource, MergeAlgorithm, OptimizeMode, RasterizeOptions};
pub use types::{AdjustedValue, GdalDataType, GdalType};

mod buffer;
pub mod dem;
#[cfg(all(major_ge_3, minor_ge_1))]
mod mdarray;
mod rasterband;
mod rasterize;
#[cfg(test)]
mod tests;
mod types;
pub mod warp;

/// Key/value pair for passing driver-specific creation options to
/// [`Driver::create_with_band_type_wth_options`](crate::Driver::create_with_band_type_with_options`).
///
/// See `papszOptions` in [GDAL's `Create(...)` API documentation](https://gdal.org/api/gdaldriver_cpp.html#_CPPv4N10GDALDriver6CreateEPKciii12GDALDataType12CSLConstList).
#[derive(Debug)]
pub struct RasterCreationOption<'a> {
    pub key: &'a str,
    pub value: &'a str,
}
