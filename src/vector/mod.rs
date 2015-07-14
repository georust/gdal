//! GDAL Vector Data
//!
//! ## Reading
//!
//! ```
//! use std::path::Path;
//! use gdal::vector::Dataset;
//!
//! let dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
//! let layer = dataset.layer(0).unwrap();
//! for feature in layer.features() {
//!     let highway_field = feature.field("highway").unwrap();
//!     let geometry = feature.geometry();
//!     println!("{} {}", highway_field.as_string(), geometry.wkt());
//! }
//! ```


pub use vector::dataset::Dataset;
pub use vector::layer::{Layer, FieldIterator, Field, FeatureIterator};
pub use vector::feature::{Feature, FieldValue};
pub use vector::geometry::Geometry;

/// Convert object to a GDAL geometry.
pub trait ToGdal {
    fn to_gdal(&self) -> Geometry;
}

mod ogr;
mod dataset;
mod layer;
mod feature;
mod geometry;
mod gdal_to_geo;
mod geo_to_gdal;

#[cfg(test)]
mod tests;
