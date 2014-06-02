use libc::{c_int, c_char, c_double};
use std::ptr::null;
use super::raster::RasterDataset;

#[link(name="gdal")]
extern {
    fn GDALReprojectImage(
        hSrcDS: *(),
        pszSrcWKT: *c_char,
        hDstDS: *(),
        pszDstWKT: *c_char,
        eResampleAlg: c_int,
        dfWarpMemoryLimit: c_double,
        dfMaxError: c_double,
        pfnProgress: *(),
        pProgressArg: *(),
        psOptions: *()
    ) -> c_int;
}

static GRA_NearestNeighbour:  c_int = 0;
static GRA_Bilinear:          c_int = 1;
static GRA_Cubic:             c_int = 2;
static GRA_CubicSpline:       c_int = 3;
static GRA_Lanczos:           c_int = 4;
static GRA_Average:           c_int = 5;
static GRA_Mode:              c_int = 6;

static REPROJECT_MEMORY_LIMIT: c_double = 0.0;

pub fn reproject(src: &RasterDataset, dst: &RasterDataset) {
    let rv = unsafe {
        GDALReprojectImage(
                src.get_ptr(),
                null(),
                dst.get_ptr(),
                null(),
                GRA_Bilinear,
                REPROJECT_MEMORY_LIMIT,
                0.0 as c_double,
                null(),
                null(),
                null()
            )
    } as int;
    assert!(rv == 0);
}
