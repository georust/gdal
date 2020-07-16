pub mod raster;
pub mod vector;
pub mod config;
pub mod version;
pub mod spatial_ref;
pub mod metadata;
pub mod dataset;
pub mod driver;
mod gdal_major_object;

use std::sync::Once;
use gdal_sys;

static START: Once = Once::new();

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            gdal_sys::GDALAllRegister();
        });
    }
}

