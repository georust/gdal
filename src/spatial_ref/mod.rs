mod srs;

pub use gdal_sys::OGRAxisOrientation;
pub use srs::{AxisOrientationType, CoordTransform, SpatialRef};

#[cfg(test)]
mod tests;
