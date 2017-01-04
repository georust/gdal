use libc::c_double;
use std::ptr::null;
use raster::{Dataset};
use raster::gdal_enums::GDALResampleAlg;
use gdal_sys::{gdal, cpl_error};
use utils::_last_cpl_err;

use errors::*;

pub fn reproject(src: &Dataset, dst: &Dataset) -> Result<()> {
    let rv = unsafe {
        gdal::GDALReprojectImage(
                src._c_ptr(),
                null(),
                dst._c_ptr(),
                null(),
                GDALResampleAlg::GRA_Bilinear,
                gdal::REPROJECT_MEMORY_LIMIT,
                0.0 as c_double,
                null(),
                null(),
                null()
            )
    };
    if rv != cpl_error::CPLErr::CE_None {            
        return Err(_last_cpl_err(rv).into());
    }
    Ok(())
}
