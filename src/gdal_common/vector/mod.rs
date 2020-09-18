//! GDAL Vector Data
//!
//! ## Reading
//!
//! ```
//! use std::path::Path;
//! use gdal::{Dataset, DatasetCommon, VectorDatasetCommon, VectorLayerCommon};
//!
//! let mut dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
//! let layer = dataset.layer(0).unwrap();
//! for feature in layer.features() {
//!     let highway_field = feature.field("highway").unwrap();
//!     let geometry = feature.geometry();
//!     println!("{} {}", highway_field.into_string().unwrap(), geometry.wkt().unwrap());
//! }
//! ```

mod dataset;
mod defn;
mod feature;
mod gdal_to_geo;
mod geo_to_gdal;
mod geometry;
mod layer;

pub use dataset::VectorDatasetCommon;
pub use defn::{Defn, Field, FieldIterator};
pub use feature::{Feature, FieldValue};
pub use gdal_sys::{OGRFieldType, OGRwkbGeometryType};
pub use geometry::Geometry;
pub use layer::{FeatureIterator, FieldDefn, Layer, VectorLayerCommon};

use crate::errors::Result;

/// Convert object to a GDAL geometry.
pub trait ToGdal {
    fn to_gdal(&self) -> Result<Geometry>;
}

#[cfg(test)]
mod tests;
