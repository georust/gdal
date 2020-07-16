//! [GDAL](http://gdal.org/) bindings for Rust.
//!
//! A high-level API to access the GDAL library, for vector and raster data.
//!
//! ## Use
//!
//! ```
//! use std::path::Path;
//! use gdal::vector::Dataset;
//!
//! let mut dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
//! let layer = dataset.layer(0).unwrap();
//! for feature in layer.features() {
//!     let highway_field = feature.field("highway").unwrap();
//!     let geometry = feature.geometry();
//!     println!("{} {}", highway_field.into_string().unwrap(), geometry.wkt().unwrap());
//! }
//! ```

#![crate_name = "gdal"]
#![crate_type = "lib"]

pub mod errors;
pub mod utils;
pub mod gdal_common;
pub use gdal_common::*;

#[cfg(feature = "gdal_2_0")]
pub mod gdal_2_0;
#[cfg(feature = "gdal_2_0")]
pub use gdal_2_0::*;

#[cfg(feature = "gdal_3_0")]
pub mod gdal_3_0;
#[cfg(feature = "gdal_3_0")]
pub use gdal_3_0::*;

#[cfg(test)]
fn assert_almost_eq(a: f64, b: f64) {
    let diff: f64 = b - a;
    assert!(diff.abs() < f64::EPSILON);
}
