//! [GDAL](http://gdal.org/) bindings for Rust.
//!
//! A high-level API to access the GDAL library, for vector and raster data.
//!
//! ## Use
//!
//! ```
//! use std::path::Path;
//! use gdal::Dataset;
//! use gdal::vector::LayerAccess;
//!
//! let dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
//! let mut layer = dataset.layer(0).unwrap();
//! for feature in layer.features() {
//!     let highway_field = feature.field("highway").unwrap().unwrap();
//!     let geometry = feature.geometry();
//!     println!("{} {}", highway_field.into_string().unwrap(), geometry.wkt().unwrap());
//! }
//! ```

#![crate_name = "gdal"]
#![crate_type = "lib"]

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
