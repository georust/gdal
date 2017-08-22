use libc::{c_int, c_char, c_double, c_void};
use ogr_enums::*;

// http://gdal.org/ogr__srs__api_8h.html

extern {
    pub fn OSRNewSpatialReference(pszWKT: *const c_char) -> *mut c_void;
    pub fn OSRClone(hSRS: *const c_void) -> *mut c_void;
    pub fn OSRDestroySpatialReference(hSRS: *mut c_void) -> c_void;
    pub fn OSRSetFromUserInput(hSRS: *mut c_void, pszDefinition: *const c_char) -> OGRErr;
    pub fn OSRImportFromEPSG(hSRS: *const c_void, nCode: c_int) -> OGRErr;
    pub fn OSRImportFromProj4(hSRS: *mut c_void, proj4_string: *const c_char) -> OGRErr;
    pub fn OSRImportFromESRI(hSRS: *mut c_void, esri_wkt: *const *const c_char) -> OGRErr;
    pub fn OSRExportToWkt(hSRS: *const c_void, ppszReturn: &mut *const c_char) -> OGRErr;
    pub fn OSRExportToPrettyWkt(hSRS: *const c_void, ppszReturn: &mut *const c_char, bSimplify: c_int) -> OGRErr;
    pub fn OSRExportToProj4(hSRS: *const c_void, ppszReturn: &mut *const c_char) -> OGRErr;
    pub fn OSRExportToXML(hSRS: *const c_void, ppszRawXML: &mut *const c_char, pszDialect: *const c_char) -> OGRErr;
    pub fn OSRMorphToESRI(hSRS: *mut c_void) -> OGRErr;
    pub fn OSRGetAuthorityCode(hSRS: *const c_void, key: *const c_char) -> *const c_char;
    pub fn OSRGetAuthorityName(hSRS: *const c_void, key: *const c_char) -> *const c_char;
    pub fn OSRIsSame(hSRS1: *const c_void, hSRS2: *const c_void) -> bool;
    pub fn OCTNewCoordinateTransformation(hSourceSRS: *const c_void, hTargetSRS: *const c_void) -> *mut c_void;
    pub fn OCTDestroyCoordinateTransformation(hCT: *mut c_void) -> c_void;
    pub fn OCTTransform(hCT: *const c_void, nCount: c_int, x: *mut c_double, y: *mut c_double, z: *mut c_double) -> bool;
}
