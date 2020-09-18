mod srs;
pub use gdal_sys::OSRAxisMappingStrategy;
pub use srs::SpatialRef_3_0;

#[cfg(test)]
mod tests;
