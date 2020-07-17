mod srs;
pub use srs::SpatialRef_3_0;
pub use gdal_sys::OSRAxisMappingStrategy;

#[cfg(test)]
mod tests;