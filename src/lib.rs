#![crate_name="gdal"]
#![crate_type="lib"]
#![feature(unsafe_destructor)]

extern crate libc;
#[cfg(test)] extern crate test;

pub use version::version_info;

mod utils;
pub mod version;
pub mod raster;
pub mod vector;
pub mod proj;
pub mod geom;

#[derive(Clone, Copy, PartialEq, Show)]
pub struct GdalError {
    pub desc: &'static str,
}
