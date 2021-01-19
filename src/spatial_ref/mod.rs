mod srs;

pub use srs::{ CoordTransform, SpatialRef, AxisOrientationType };
pub use gdal_sys::{ OGRAxisOrientation };

#[cfg(test)]
mod tests;
