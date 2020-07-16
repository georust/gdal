pub mod raster;
pub mod vector;
pub mod config;
mod version;
mod spatial_ref;
mod metadata;
mod dataset;
mod driver;
mod gdal_major_object;

pub use {dataset::*, driver::*, metadata::*, spatial_ref::*, version::*};

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

