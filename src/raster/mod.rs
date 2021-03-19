//! GDAL Raster Data

mod rasterband;
mod types;
mod warp;

pub use rasterband::{Buffer, ByteBuffer, ColorInterpretation, RasterBand};
pub use types::{GDALDataType, GdalType};
pub use warp::{create_and_reproject, reproject};

#[cfg(test)]
mod tests;
