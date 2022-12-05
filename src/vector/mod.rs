//! GDAL Vector Data API
//!
//! This example opens a vector [`Dataset`](crate::Dataset) and iterates over the various levels of structure within it.
//! The GDAL vector data model is quite sophisticated, so please refer to the GDAL
//! [Vector Data Model](https://gdal.org/user/vector_data_model.html) document for specifics.
//!
//! ```rust, no_run
//! use gdal::{Dataset, Metadata};
//! // The `LayerAccess` trait enables reading of vector specific fields from the `Dataset`.
//! use gdal::vector::LayerAccess;
//! # fn main() -> gdal::errors::Result<()> {
//! use gdal::vector::geometry_type_to_name;
//! let dataset = Dataset::open("fixtures/roads.geojson")?;
//! println!("Dataset description: {}", dataset.description()?);
//! let layer_count = dataset.layer_count();
//! println!("Number of layers: {layer_count}");
//! // Unlike raster bands, layers are zero-based
//! for l in 0..layer_count {
//!     // We have to get a mutable borrow on the layer because the `Layer::features` iterator
//!     // requires it.
//!     let mut layer = dataset.layer(l)?;
//!     let feature_count = layer.feature_count();
//!     println!("  Layer {l}, name='{}', features={}", layer.name(), feature_count);
//!     for feature in layer.features() {
//!         // The fid is important in cases where the vector dataset is large can you
//!         // need random access.
//!         let fid = feature.fid().unwrap_or(0);
//!         // Summarize the geometry
//!         let geometry = feature.geometry().unwrap();
//!         let geom_type = geometry_type_to_name(geometry.geometry_type());
//!         let geom_len = geometry.get_point_vec().len();
//!         println!("    Feature fid={fid:?}, geometry_type='{geom_type}', geometry_len={geom_len}");
//!         // Get all the available fields and print their values
//!         for field in feature.fields() {
//!             let name = field.0;
//!             let value = field.1.and_then(|f| f.into_string()).unwrap_or("".into());
//!             println!("      {name}={value}");
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! The resulting (truncated) output looks like this:
//!
//! ```text
//! Dataset description: fixtures/roads.geojson
//! Number of layers: 1
//!   Layer 0, name='roads', features=21
//!     Feature fid=236194095, geometry_type='Line String', geometry_len=3
//!       kind=path
//!       sort_key=
//!       is_link=no
//!       is_tunnel=no
//!       is_bridge=no
//!       railway=
//!       highway=footway
//!     Feature fid=236194098, geometry_type='Line String', geometry_len=3
//!       ...
//! ...
//! ```
//!

mod defn;
mod feature;
mod gdal_to_geo;
mod geo_to_gdal;
mod geometry;
mod layer;
mod ops;
pub mod sql;

pub use defn::{Defn, Field, FieldIterator};
pub use feature::{field_type_to_name, Feature, FieldValue, FieldValueIterator};
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
