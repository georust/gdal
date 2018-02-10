//! GDAL Raster Data

pub use raster::dataset::{Dataset, Buffer, ByteBuffer};
pub use raster::driver::Driver;
pub use raster::warp::reproject;
pub use raster::rasterband::{RasterBand};
pub use raster::types::GdalType;

pub use gdal_sys::gdal_enums;
pub use gdal_sys::cpl_error::{CPLErr};

pub mod types;
pub mod dataset;
pub mod driver;
pub mod warp;
pub mod rasterband;

#[cfg(test)]
mod tests;
