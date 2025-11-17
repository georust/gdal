use std::ptr::{null, null_mut};

use gdal_sys::{CPLErr, GDALResampleAlg};

use crate::dataset::Dataset;
use crate::errors::*;
use crate::utils::_last_cpl_err;

pub fn reproject(src: &Dataset, dst: &Dataset) -> Result<()> {
    let rv = unsafe {
        gdal_sys::GDALReprojectImage(
            src.c_dataset(),
            null(),
            dst.c_dataset(),
            null(),
            GDALResampleAlg::GRA_Bilinear,
            0.0,
            0.0,
            None,
            null_mut(),
            null_mut(),
        )
    };
    if rv != CPLErr::CE_None {
        return Err(_last_cpl_err(rv));
    }
    Ok(())
}
