//! GDAL Spatial Reference System Functions
//!
//! <https://gdal.org/api/ogr_srs_api.html>

mod srs;
mod transform;
mod transform_opts;

pub use srs::{AxisOrientationType, SpatialRef};
pub use transform::CoordTransform;
pub use transform_opts::CoordTransformOptions;
