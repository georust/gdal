#![crate_name = "gdal"]
#![crate_type = "lib"]
#![doc = include_str!("../README.md")]

//! ## Show Me Code!
//!
//! To get you started with GDAL (without having to read the whole manual!),
//! take a look at the examples in the [`raster`](raster#example) and [`vector`](vector#example) modules.
//!
//! ## Data Model
//!
//! At the top level, GDAL uses the same data model to access both vector and raster data sets.
//! There are several shared data model constructs at this level, but the first ones to become
//! familiar with are [`Driver`], [`Dataset`], and [`Metadata`].
//!
//! ### Driver
//!
//! One of GDAL's major strengths is the vast number of data formats it's able to work with.
//! The GDAL Manual has a full list of available [raster](https://gdal.org/drivers/raster/index.html)
//! and [vector](https://gdal.org/drivers/vector/index.html) drivers.
//!
//! The [`Driver` API][Driver] provides the requisite access points for working GDAL's drivers.
//!
//! ### Dataset
//!
//! [`Dataset`] is the top-level container for accessing all data within a data set, whether raster or vector.
//! Some methods and traits on `Dataset` are shared between raster and vector datasets,
//! and (due to historical reasons) some associated functions are only applicable to one context or the other.
//! The [`raster`] and [`vector`] modules cover these specifics.
//!
//! ### Metadata
//!
//! Metadata in GDAL takes a number of forms, some of which are specific to purpose
//! (e.g. pixel interpretation, spatial reference system),
//! and other more general-purpose (e.g. acquisition date-time). The former will be covered in
//! relevant sections of the [`raster`] and [`vector`] modules, and the general-purpose data model
//! in the [`Metadata`] API.
//!

pub use version::version_info;

pub mod config;
pub mod cpl;
mod dataset;
mod driver;
pub mod errors;
mod gdal_major_object;
mod metadata;
pub mod programs;
pub mod raster;
pub mod spatial_ref;
#[cfg(test)]
pub mod test_utils;
mod utils;
pub mod vector;
pub mod version;
pub mod vsi;

pub use dataset::{
    Dataset, DatasetOptions, GdalOpenFlags, GeoTransform, GeoTransformEx, LayerIterator,
    LayerOptions, Transaction,
};
pub use driver::Driver;
pub use metadata::Metadata;

#[cfg(test)]
fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.00001);
    assert!(f > 0.99999);
}
