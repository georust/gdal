use crate::utils::_last_cpl_err;
use crate::{dataset::Dataset, spatial_ref::SpatialRef, Driver};
use gdal_sys::{self, CPLErr, GDALResampleAlg};
use std::{
    ffi::CString,
    path::Path,
    ptr::{null, null_mut},
};

use crate::errors::*;

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

pub fn create_and_reproject(
    src: &Dataset,
    dst_path: &Path,
    dst_srs: &SpatialRef,
    dst_driver: &Driver,
) -> Result<()> {
    let psz_dst_filename = dst_path
        .to_str()
        .expect("Destination path must be supplied.");
    let psz_dst_wkt = dst_srs.to_wkt().expect("Failed to obtain WKT for SRS.");

    let rv = unsafe {
        gdal_sys::GDALCreateAndReprojectImage(
            src.c_dataset(),
            null(),
            CString::new(psz_dst_filename)?.as_ptr(),
            CString::new(psz_dst_wkt)?.as_ptr(),
            dst_driver.c_driver(),
            null_mut(),
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
