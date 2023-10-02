//! GDAL Spatial Reference System
//!
//! See: [Spatial Reference System C API](https://gdal.org/api/ogr_srs_api.html).
//!
//! See also: [OGR Coordinate Reference Systems and Coordinate Transformation Tutorial](https://gdal.org/tutorials/osr_api_tut.html)

mod srs;
mod transform;
mod transform_opts;

/// Axis orientation options
///
/// See [`OGRAxisOrientation`](https://gdal.org/api/ogr_srs_api.html#_CPPv418OGRAxisOrientation).
pub type AxisOrientationType = gdal_sys::OGRAxisOrientation::Type;

pub use srs::{SpatialRef, SpatialRefRef};
pub use transform::{CoordTransform, CoordTransformRef};
pub use transform_opts::{CoordTransformOptions, CoordTransformOptionsRef};
