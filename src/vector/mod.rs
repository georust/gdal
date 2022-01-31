//! GDAL Vector Data
//!
//! ## Reading
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

mod defn;
mod feature;
mod gdal_to_geo;
mod geo_to_gdal;
mod geometry;
mod layer;
mod ops;
pub mod sql;

pub use defn::{Defn, Field, FieldIterator};
pub use feature::{Feature, FieldValue, FieldValueIterator};
pub use gdal_sys::{OGRFieldType, OGRwkbGeometryType};
pub use geometry::{geometry_type_to_name, Geometry};
pub use layer::{
    FeatureIterator, FieldDefn, Layer, LayerAccess, LayerCaps, OwnedFeatureIterator, OwnedLayer,
};
pub use ops::GeometryIntersection;

use crate::errors::Result;

/// Convert object to a GDAL geometry.
pub trait ToGdal {
    fn to_gdal(&self) -> Result<Geometry>;
}

#[cfg(test)]
mod vector_tests;
