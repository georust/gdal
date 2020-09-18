pub mod config;

mod dataset;
mod driver;
mod gdal_major_object;
mod metadata;
mod raster;
mod spatial_ref;
mod vector;
mod version;

pub use {dataset::*, driver::*, metadata::*, raster::*, spatial_ref::*, vector::*, version::*};

use gdal_sys;
use std::sync::Once;

static START: Once = Once::new();

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            gdal_sys::GDALAllRegister();
        });
    }
}
