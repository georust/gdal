use libc::{c_int, c_char, c_double, c_void};
use super::gdal_enums::*;

#[link(name="gdal")]
extern {
    // driver
    pub fn GDALAllRegister();
    pub fn GDALGetDriverByName(pszName: *const c_char) -> *const c_void;
    pub fn GDALGetDriverShortName(hDriver: *const c_void) -> *const c_char;
    pub fn GDALGetDriverLongName(hDriver: *const c_void) -> *const c_char;
    pub fn GDALCreate(
            hDriver: *const c_void,
            pszFilename: *const c_char,
            nXSize: c_int,
            nYSize: c_int,
            nBands: c_int,
            eBandType: GDALDataType,
            papszOptions: *const *const c_char
        ) -> *const c_void;
    pub fn GDALCreateCopy(
            hDriver: *const c_void,
            pszFilename: *const c_char,
            hSrcDS: *const c_void,
            bStrict: c_int,
            papszOptions: *const *const c_char,
            pfnProgres: *const c_void,
            pProgressData: *const c_void
        ) -> *const c_void;
    pub fn GDALOpen(pszFilename: *const c_char, eAccess: GDALAccess) -> *const c_void;
    // dataset
    pub fn GDALClose(hDS: *const c_void);
    pub fn GDALGetDatasetDriver(hDataset: *const c_void) -> *const c_void;
    pub fn GDALGetRasterXSize(hDataset: *const c_void) -> c_int;
    pub fn GDALGetRasterYSize(hDataset: *const c_void) -> c_int;
    pub fn GDALGetRasterCount(hDataset: *const c_void) -> c_int;
    pub fn GDALGetProjectionRef(hDataset: *const c_void) -> *const c_char;
    pub fn GDALSetProjection(hDataset: *const c_void, pszProjection: *const c_char) -> c_int;
    pub fn GDALSetGeoTransform(hDataset: *const c_void, padfTransform: *const c_double) -> c_int;
    pub fn GDALGetGeoTransform(hDataset: *const c_void, padfTransform: *mut c_double) -> c_int;
    pub fn GDALGetRasterBand(hDataset: *const c_void, nBandId: c_int) -> *const c_void;
    // band
    pub fn GDALGetRasterDataType(hBand: *const c_void) -> c_int;
    pub fn GDALGetRasterNoDataValue(hBand: *const c_void, pbSuccess: *mut c_int) -> c_double;
    pub fn GDALGetRasterOffset(hBand: *const c_void, pbSuccess: *mut c_int) -> c_double;
    pub fn GDALGetRasterScale(hBand: *const c_void, pbSuccess: *mut c_int) -> c_double;
    pub fn GDALRasterIO(
            hBand: *const c_void,
            eRWFlag: GDALRWFlag,
            nXOff: c_int,
            nYOff: c_int,
            nXSize: c_int,
            nYSize: c_int,
            pData: *const c_void,
            nBufXSize: c_int,
            nBufYSize: c_int,
            GDALDataType: GDALDataType,
            nPixelSpace: c_int,
            nLineSpace: c_int
        ) -> c_int;
    pub fn GDALReprojectImage(
        hSrcDS: *const c_void,
        pszSrcWKT: *const c_char,
        hDstDS: *const c_void,
        pszDstWKT: *const c_char,
        eResampleAlg: GDALResampleAlg,
        dfWarpMemoryLimit: c_double,
        dfMaxError: c_double,
        pfnProgress: *const c_void,
        pProgressArg: *const c_void,
        psOptions: *const c_void
    ) -> c_int;
}

pub static REPROJECT_MEMORY_LIMIT: c_double = 0.0;
