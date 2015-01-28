pub use raster::dataset::{RasterDataset, ByteBuffer, open};
pub use raster::driver::{Driver, driver};

mod gdal;
pub mod dataset;
pub mod driver;

#[cfg(test)]
mod tests;
