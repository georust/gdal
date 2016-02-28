use libc::{c_int, c_char, c_double, c_void};

#[link(name="gdal")]
extern {
    pub fn OGRRegisterAll();
    pub fn OGRGetDriverByName(pszName: *const c_char) -> *const c_void;
    pub fn OGR_Dr_CreateDataSource(hDriver: *const c_void, pszName: *const c_char, papszOptions: *const c_void) -> *const c_void;
    pub fn OGROpen(pszName: *const c_char, bUpdate: c_int, pahDriverList: *const c_void) -> *const c_void;
    pub fn OGR_DS_GetLayerCount(hDS: *const c_void) -> c_int;
    pub fn OGR_DS_Destroy(hDataSource: *const c_void);
    pub fn OGR_DS_GetLayer(hDS: *const c_void, iLayer: c_int) -> *const c_void;
    pub fn OGR_DS_CreateLayer(hDS: *const c_void, pszName: *const c_char, hSpatialRef: *const c_char, eType: c_int, papszOptions: *const c_void) -> *const c_void;
    pub fn OGR_L_GetLayerDefn(hLayer: *const c_void) -> *const c_void;
    pub fn OGR_L_GetNextFeature(hLayer: *const c_void) -> *const c_void;
    pub fn OGR_L_SetSpatialFilter(hLayer: *const c_void, hGeom: *const c_void);
    pub fn OGR_L_CreateFeature(hLayer: *const c_void, hFeat: *const c_void) -> c_int;
    pub fn OGR_FD_GetFieldCount(hDefn: *const c_void) -> c_int;
    pub fn OGR_FD_GetFieldDefn(hDefn: *const c_void, iField: c_int) -> *const c_void;
    pub fn OGR_F_Create(hDefn: *const c_void) -> *const c_void;
    pub fn OGR_F_GetFieldIndex(hFeat: *const c_void, pszName: *const c_char) -> c_int;
    pub fn OGR_F_GetFieldDefnRef(hFeat: *const c_void, i: c_int) -> *const c_void;
    pub fn OGR_F_GetFieldAsString(hFeat: *const c_void, iField: c_int) -> *const c_char;
    pub fn OGR_F_GetFieldAsDouble(hFeat: *const c_void, iField: c_int) -> c_double;
    pub fn OGR_F_GetGeometryRef(hFeat: *const c_void) -> *const c_void;
    pub fn OGR_F_SetGeometryDirectly(hFeat: *const c_void, hGeom: *const c_void) -> c_int;
    pub fn OGR_F_Destroy(hFeat: *const c_void);
    pub fn OGR_G_CreateGeometry(eGeometryType: c_int) -> *const c_void;
    pub fn OGR_G_CreateFromWkt(ppszData: &mut *const c_char, hSRS: *const c_void, phGeometry: &mut *const c_void) -> c_int;
    pub fn OGR_G_GetGeometryType(hGeom: *const c_void) -> c_int;
    pub fn OGR_G_GetPoint(hGeom: *const c_void, i: c_int, pdfX: &mut c_double, pdfY: &mut c_double, pdfZ: &mut c_double);
    pub fn OGR_G_GetPointCount(hGeom: *const c_void) -> c_int;
    pub fn OGR_G_SetPoint_2D(hGeom: *const c_void, i: c_int, dfX: c_double, dfY: c_double);
    pub fn OGR_G_ExportToWkt(hGeom: *const c_void, ppszSrcText: &mut *const c_char) -> c_int;
    pub fn OGR_G_ExportToJson(hGeometry: *const c_void) -> *const c_char;
    pub fn OGR_G_ConvexHull(hTarget: *const c_void) -> *const c_void;
    pub fn OGR_G_GetGeometryCount(hGeom: *const c_void) -> c_int;
    pub fn OGR_G_GetGeometryRef(hGeom: *const c_void, iSubGeom: c_int) -> *const c_void;
    pub fn OGR_G_AddGeometryDirectly(hGeom: *const c_void, hNewSubGeom: *const c_void) -> c_int;
    pub fn OGR_G_DestroyGeometry(hGeom: *mut c_void);
    pub fn OGR_Fld_GetNameRef(hDefn: *const c_void) -> *const c_char;
    pub fn OGR_Fld_GetType(hDefn: *const c_void) -> c_int;
    pub fn OGRFree(ptr: *mut c_void);
    pub fn VSIFree(ptr: *mut c_void);
}

pub const OGRERR_NONE:            c_int = 0;

pub const OFT_REAL:               c_int = 2;
pub const OFT_STRING:             c_int = 4;

pub const WKB_UNKNOWN:            c_int = 0;
pub const WKB_POINT:              c_int = 1;
pub const WKB_LINESTRING:         c_int = 2;
pub const WKB_POLYGON:            c_int = 3;
pub const WKB_MULTIPOINT:         c_int = 4;
pub const WKB_MULTILINESTRING:    c_int = 5;
pub const WKB_MULTIPOLYGON:       c_int = 6;
pub const WKB_GEOMETRYCOLLECTION: c_int = 7;
pub const WKB_LINEARRING:         c_int = 101;
