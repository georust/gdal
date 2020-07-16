//! GDAL Vector Data
//!
//! ## Reading
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

pub use crate::vector::dataset::VectorDatasetCommon;
pub use crate::vector::defn::{Defn, Field, FieldIterator};
pub use crate::driver::Driver;
pub use crate::vector::feature::{Feature, FieldValue};
pub use crate::vector::geometry::Geometry;
pub use crate::vector::layer::{FeatureIterator, FieldDefn, Layer, VectorLayerCommon};
pub use gdal_sys::{OGRFieldType, OGRwkbGeometryType};

use crate::errors::Result;

/// Convert object to a GDAL geometry.
pub trait ToGdal {
    fn to_gdal(&self) -> Result<Geometry>;
}

mod dataset;
mod defn;
mod feature;
mod gdal_to_geo;
mod geo_to_gdal;
mod geometry;
mod layer;

#[cfg(test)]
mod tests;
