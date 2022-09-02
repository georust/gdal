//! GDAL Raster Data

#[cfg(all(major_ge_3, minor_ge_1))]
mod mdarray;
mod rasterband;
mod rasterize;
mod types;
mod warp;

#[cfg(all(major_ge_3, minor_ge_1))]
pub use mdarray::{ExtendedDataType, ExtendedDataTypeClass, Group, MDArray};
pub use rasterband::{
    Buffer, ByteBuffer, CmykEntry, ColorEntry, ColorInterpretation, ColorTable, GrayEntry,
    HlsEntry, PaletteInterpretation, RasterBand, ResampleAlg, RgbaEntry,
};
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
