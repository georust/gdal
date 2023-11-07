//! GDAL Warp processing routines.

use std::path::Path;

pub use options::*;

use crate::errors::Result;
use crate::raster::processing::warp::reproject::ReprojectOptions;
use crate::spatial_ref::SpatialRef;
use crate::Dataset;

mod options;
mod reproject;
mod resample;

pub trait WarpProcessing {
    /// Reproject a dataset into a new projection
    ///
    /// See [`ReprojectOptions`] for additional options.
    fn reproject<P: AsRef<Path>>(
        &self,
        dst_file: P,
        dst_projection: &SpatialRef,
        options: &ReprojectOptions,
    ) -> Result<()>;
}

impl WarpProcessing for Dataset {
    fn reproject<P: AsRef<Path>>(
        &self,
        dst_file: P,
        dst_projection: &SpatialRef,
        options: &ReprojectOptions,
    ) -> Result<()> {
        let dest_file = dst_file.as_ref();
        reproject::reproject(self, dest_file, dst_projection, &options)
    }
}
