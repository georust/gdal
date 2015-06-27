#![crate_name="gdal"]
#![crate_type="lib"]

extern crate libc;

pub use version::version_info;

mod utils;
pub mod version;
pub mod raster;
pub mod vector;
pub mod proj;
pub mod geom;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GdalError {
    pub desc: &'static str,
}
