use libc::{c_int, c_char, c_double};
use super::gdal_enums::*;

#[link(name="gdal")]
extern {
    pub fn GDALAllRegister();
    pub fn GDALGetDriverByName(pszName: *const c_char) -> *const ();
    pub fn GDALGetDriverShortName(hDriver: *const ()) -> *const c_char;
    pub fn GDALGetDriverLongName(hDriver: *const ()) -> *const c_char;
    pub fn GDALCreate(
            hDriver: *const (),
            pszFilename: *const c_char,
            nXSize: c_int,
            nYSize: c_int,
            nBands: c_int,
            eBandType: GDALDataType,
            papszOptions: *const *const c_char
        ) -> *const ();
    pub fn GDALCreateCopy(
            hDriver: *const (),
            pszFilename: *const c_char,
            hSrcDS: *const (),
            bStrict: c_int,
            papszOptions: *const *const c_char,
            pfnProgres: *const (),
            pProgressData: *const ()
        ) -> *const ();
    pub fn GDALOpen(pszFilename: *const c_char, eAccess: GDALAccess) -> *const ();
    pub fn GDALClose(hDS: *const ());
    pub fn GDALGetDatasetDriver(hDataset: *const ()) -> *const ();
    pub fn GDALGetRasterXSize(hDataset: *const ()) -> c_int;
    pub fn GDALGetRasterYSize(hDataset: *const ()) -> c_int;
    pub fn GDALGetRasterCount(hDataset: *const ()) -> c_int;
    pub fn GDALGetRasterDataType(hBand: *const()) -> c_int;
    pub fn GDALGetProjectionRef(hDS: *const ()) -> *const c_char;
    pub fn GDALSetProjection(hDS: *const (), pszProjection: *const c_char) -> c_int;
    pub fn GDALSetGeoTransform(hDS: *const (), padfTransform: *const c_double) -> c_int;
    pub fn GDALGetGeoTransform(hDS: *const (), padfTransform: *mut c_double) -> c_int;
    pub fn GDALGetRasterBand(hDS: *const (), nBandId: c_int) -> *const ();
    pub fn GDALRasterIO(
            hBand: *const (),
            eRWFlag: GDALRWFlag,
            nXOff: c_int,
            nYOff: c_int,
            nXSize: c_int,
            nYSize: c_int,
            pData: *const (),
            nBufXSize: c_int,
            nBufYSize: c_int,
            GDALDataType: GDALDataType,
            nPixelSpace: c_int,
            nLineSpace: c_int
        ) -> c_int;
    pub fn GDALReprojectImage(
        hSrcDS: *const (),
        pszSrcWKT: *const c_char,
        hDstDS: *const (),
        pszDstWKT: *const c_char,
        eResampleAlg: GDALResampleAlg,
        dfWarpMemoryLimit: c_double,
        dfMaxError: c_double,
        pfnProgress: *const (),
        pProgressArg: *const (),
        psOptions: *const ()
    ) -> c_int;
}

pub static REPROJECT_MEMORY_LIMIT: c_double = 0.0;
