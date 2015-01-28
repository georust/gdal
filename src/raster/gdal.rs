use libc::{c_int, c_char, c_double};

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
            eBandType: c_int,
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
    pub fn GDALOpen(pszFilename: *const c_char, eAccess: c_int) -> *const ();
    pub fn GDALClose(hDS: *const ());
    pub fn GDALGetDatasetDriver(hDataset: *const ()) -> *const ();
    pub fn GDALGetRasterXSize(hDataset: *const ()) -> c_int;
    pub fn GDALGetRasterYSize(hDataset: *const ()) -> c_int;
    pub fn GDALGetRasterCount(hDataset: *const ()) -> c_int;
    pub fn GDALGetProjectionRef(hDS: *const ()) -> *const c_char;
    pub fn GDALSetProjection(hDS: *const (), pszProjection: *const c_char) -> c_int;
    pub fn GDALSetGeoTransform(hDS: *const (), padfTransform: *const c_double) -> c_int;
    pub fn GDALGetGeoTransform(hDS: *const (), padfTransform: *mut c_double) -> c_int;
    pub fn GDALGetRasterBand(hDS: *const (), nBandId: c_int) -> *const ();
    pub fn GDALRasterIO(
            hBand: *const (),
            eRWFlag: c_int,
            nXOff: c_int,
            nYOff: c_int,
            nXSize: c_int,
            nYSize: c_int,
            pData: *const (),
            nBufXSize: c_int,
            nBufYSize: c_int,
            GDALDataType: c_int,
            nPixelSpace: c_int,
            nLineSpace: c_int
        ) -> c_int;
}

pub const GA_READONLY:  c_int = 0;
pub const GDT_BYTE:     c_int = 1;
pub const GF_READ:      c_int = 0;
pub const GF_WRITE:     c_int = 1;
