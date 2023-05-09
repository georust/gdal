//! GDAL Spatial Reference System Functions
//!
//! <https://gdal.org/api/ogr_srs_api.html>

mod srs;
mod transform_opts;

pub use srs::{AxisOrientationType, CoordTransform, SpatialRef};
pub use transform_opts::CoordTransformOptions;