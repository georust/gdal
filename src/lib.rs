#![crate_name="gdal"]
#![crate_type="lib"]
#![feature(convert)]
#![feature(std_misc)]
#![cfg_attr(test, feature(test))]
#![cfg_attr(test, feature(collections))]

extern crate libc;

#[cfg(test)]
extern crate test;

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
