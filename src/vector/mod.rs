//! GDAL Vector Data
//!
//! ## Reading
//!
//! ```
//! use std::path::Path;
//! use gdal::Dataset;
//!
//! let mut dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
//! let layer = dataset.layer(0).unwrap();
//! for feature in layer.features() {
//!     let highway_field = feature.field("highway").unwrap().unwrap();
//!     let geometry = feature.geometry();
//!     println!("{} {}", highway_field.into_string().unwrap(), geometry.wkt().unwrap());
//! }
//! ```

mod defn;
mod feature;
mod gdal_to_geo;
mod geo_to_gdal;
mod geometry;
mod layer;
mod ops;

pub use defn::{Defn, Field, FieldIterator};
pub use feature::{Feature, FieldValue, FieldValueIterator};
pub use gdal_sys::{OGRFieldType, OGRwkbGeometryType};
pub use geometry::Geometry;
pub use layer::{FeatureIterator, FieldDefn, Layer};
pub use ops::GeometryIntersection;

use crate::errors::Result;

/// Convert object to a GDAL geometry.
pub trait ToGdal {
    fn to_gdal(&self) -> Result<Geometry>;
}

#[cfg(test)]
mod vector_tests;
