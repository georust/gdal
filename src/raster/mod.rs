//! GDAL Raster Data

pub use raster::dataset::{Dataset, Buffer, ByteBuffer};
pub use raster::driver::Driver;
pub use raster::warp::reproject;
pub use raster::rasterband::{RasterBand};

use gdal_sys::gdal;
pub use gdal_sys::gdal_enums;

mod types;
pub mod dataset;
pub mod driver;
pub mod warp;
pub mod rasterband;

#[cfg(test)]
mod tests;
