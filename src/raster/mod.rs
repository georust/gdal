//! GDAL Raster Data

mod rasterband;
mod rasterize;
mod types;
mod warp;

pub use rasterband::{Buffer, ByteBuffer, ColorInterpretation, RasterBand, ResampleAlg};
pub use rasterize::{rasterize, BurnSource, MergeAlgorithm, OptimizeMode, RasterizeOptions};
pub use types::{GDALDataType, GdalType};
pub use warp::reproject;

#[derive(Debug)]
pub struct RasterCreationOption<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[cfg(test)]
mod tests;
