pub mod config;

mod raster;
mod vector;
mod version;
mod spatial_ref;
mod metadata;
mod dataset;
mod driver;
mod gdal_major_object;

pub use {dataset::*, driver::*, metadata::*, spatial_ref::*, version::*, raster::*, vector::*};

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

