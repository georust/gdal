use crate::dataset::Dataset;
use crate::utils::_last_cpl_err;
use foreign_types::ForeignType;
use gdal_sys::{self, CPLErr, GDALResampleAlg};
use std::ptr::{null, null_mut};

use crate::errors::*;

pub fn reproject(src: &Dataset, dst: &Dataset) -> Result<()> {
    let rv = unsafe {
        gdal_sys::GDALReprojectImage(
            src.as_ptr(),
            null(),
            dst.as_ptr(),
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
