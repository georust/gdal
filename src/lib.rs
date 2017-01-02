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
//!     println!("{} {}", highway_field.as_string(), geometry.wkt());
//! }
//! ```

#![crate_name="gdal"]
#![crate_type="lib"]


extern crate libc;
extern crate geo;
extern crate gdal_sys;

pub use version::version_info;

mod utils;
mod gdal_major_object;
pub mod metadata;
pub mod version;
pub mod raster;
pub mod vector;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GdalError {
    pub desc: &'static str,
}
