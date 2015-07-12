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
pub use vector::layer::{Layer, FeatureIterator};
pub use vector::feature::Feature;
pub use vector::geometry::{Geometry, ToGdal};

mod ogr;
pub mod dataset;
pub mod layer;
pub mod feature;
pub mod geometry;

#[cfg(test)]
mod tests;
