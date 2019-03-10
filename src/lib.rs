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

extern crate failure;
extern crate gdal_sys;
extern crate geo_types;
extern crate libc;
extern crate num_traits;
extern crate ndarray;

pub use version::version_info;

pub mod config;
pub mod errors;
mod gdal_major_object;
pub mod metadata;
pub mod raster;
pub mod spatial_ref;
mod utils;
pub mod vector;
pub mod version;

#[cfg(test)]
fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.00001);
    assert!(f > 0.99999);
}
