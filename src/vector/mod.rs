//! # GDAL Vector Data
//!
//! ## Reading
//!
//! ```
//! use std::path::Path;
//! use gdal::vector::Dataset;
//!
//! let ds = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
//! let layer = ds.layer(0).unwrap();
//! for f in layer.features() {
//!     println!("{} {}", f.field("highway").unwrap().as_string(), f.wkt());
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
pub mod geom;

#[cfg(test)]
mod tests;
