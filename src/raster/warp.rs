use libc::c_double;
use std::ptr::{null, null_mut};
use raster::{Dataset};
use gdal_sys::{self, CPLErr, GDALResampleAlg};
use utils::_last_cpl_err;

use errors::*;

pub fn reproject(src: &Dataset, dst: &Dataset) -> Result<()> {
    let rv = unsafe {
        gdal_sys::GDALReprojectImage(
                src._c_ptr(),
                null(),
                dst._c_ptr(),
                null(),
                GDALResampleAlg::GRA_Bilinear,
                0.0,
                0.0 as c_double,
                None,
                null_mut(),
                null_mut()
            )
    };
    if rv != CPLErr::CE_None {
        Err(_last_cpl_err(rv))?;
    }
    Ok(())
}
