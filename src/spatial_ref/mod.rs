//! GDAL Spatial Reference System Functions
//!
//! https://gdal.org/api/ogr_srs_api.html

mod srs;

pub use gdal_sys::OGRAxisOrientation;
pub use srs::{AxisOrientationType, CoordTransform, SpatialRef};

#[cfg(test)]
mod tests;
