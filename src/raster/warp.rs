use libc::c_double;
use std::ptr::null;
use raster::{Dataset};
use raster::gdal_enums::GDALResampleAlg;
use gdal_sys::gdal;

pub fn reproject(src: &Dataset, dst: &Dataset) {
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
    } as isize;
    assert!(rv == 0);
}
