//! GDAL Raster Data

pub use dataset::{RasterDatasetCommon};
pub use rasterband::{RasterBand, RasterBandCommon};
pub use warp::reproject;
pub use types::GdalType;

mod dataset;
mod rasterband;
mod types;
mod warp;

#[cfg(test)]
mod tests;

pub struct RasterBuffer<T: GdalType> {
    pub size: (usize, usize),
    pub data: Vec<T>,
}

impl<T: GdalType> RasterBuffer<T> {
    pub fn new(size: (usize, usize), data: Vec<T>) -> RasterBuffer<T> {
        RasterBuffer { size, data }
    }
}