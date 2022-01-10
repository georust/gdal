//! [GDAL](http://gdal.org/) bindings for Rust.
//!
//! A high-level API to access the GDAL library, for vector and raster data.
//!
//! ## Use
//!
//! ```
//! use std::path::Path;
//! use gdal::Dataset;
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
pub mod raster;
pub mod raster_programs;
pub mod spatial_ref;
mod utils;
pub mod vector;
pub mod version;
pub mod vsi;

pub use dataset::{
    Dataset, DatasetOptions, GdalOpenFlags, GeoTransform, LayerIterator, LayerOptions, Transaction,
};
pub use driver::Driver;
pub use metadata::Metadata;

/// Apply GeoTransform to x/y coordinate.
/// Wraps [GDALApplyGeoTransform].
/// 
/// [GDALApplyGeoTransform]: https://gdal.org/api/raster_c_api.html#_CPPv421GDALApplyGeoTransformPdddPdPd
pub fn apply_geo_transform(geo_transform: &GeoTransform, pixel: f64, line: f64) -> (f64, f64) {
    let mut geo_x: f64 = 0.;
    let mut geo_y: f64 = 0.;
    unsafe {
        gdal_sys::GDALApplyGeoTransform(geo_transform.as_ptr() as *mut f64, pixel, line, &mut geo_x, &mut geo_y);
    }
    (geo_x, geo_y)
}

#[cfg(test)]
fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.00001);
    assert!(f > 0.99999);
}
