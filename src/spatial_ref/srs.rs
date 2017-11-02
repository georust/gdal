use libc::{c_int, c_char, c_void};
use std::ffi::{CString, CStr};
use std::ptr;
use std::str::FromStr;
use utils::{_string, _last_null_pointer_err};
use gdal_sys::{osr, ogr_enums};
use gdal_sys::ogr_enums::OGRErr;

use errors::*;

pub struct CoordTransform(*mut c_void);

impl Drop for CoordTransform {
    fn drop(&mut self) {
        unsafe { osr::OCTDestroyCoordinateTransformation(self.0) };
        self.0 = ptr::null_mut();
    }
}

impl CoordTransform {
    pub fn new(sp_ref1: &SpatialRef, sp_ref2: &SpatialRef) -> Result<CoordTransform> {
        let c_obj = unsafe { osr::OCTNewCoordinateTransformation(sp_ref1.0, sp_ref2.0) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OCTNewCoordinateTransformation").into());
        }
        Ok(CoordTransform(c_obj))
    }

    pub fn transform_coord(&self, x: &mut [f64], y: &mut [f64], z: &mut [f64]){
        let nb_coords = x.len();
        assert_eq!(nb_coords, y.len());
        let ret_val = unsafe { osr::OCTTransform(
            self.0,
            nb_coords as c_int,
            x.as_mut_ptr(),
            y.as_mut_ptr(),
            z.as_mut_ptr()
        ) };
        assert_eq!(true, ret_val);
    }

    pub fn to_c_hct(&self) -> *const c_void {
        self.0 as *const c_void
    }
}

#[derive(Debug)]
pub struct SpatialRef(*mut c_void);

impl Drop for SpatialRef {
    fn drop(&mut self){
        unsafe { osr::OSRRelease(self.0)};
        self.0 = ptr::null_mut();
    }
}

impl Clone for SpatialRef {
    fn clone(&self) -> SpatialRef {
        let n_obj = unsafe { osr::OSRClone(self.0 as *const c_void)};
        SpatialRef(n_obj)
    }
}

impl PartialEq for SpatialRef {
    fn eq(&self, other: &SpatialRef) -> bool {
        unsafe { osr::OSRIsSame(self.0, other.0)}
    }
}

impl SpatialRef {
    pub fn new() -> Result<SpatialRef> {
        let c_obj = unsafe { osr::OSRNewSpatialReference(ptr::null()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference").into());
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_definition(definition: &str) -> Result<SpatialRef> {
        let c_obj = unsafe { osr::OSRNewSpatialReference(ptr::null()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference").into());
        }
        let rv = unsafe { osr::OSRSetFromUserInput(c_obj, CString::new(definition)?.as_ptr()) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OSRSetFromUserInput").into());
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_wkt(wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(wkt)?;
        let c_obj = unsafe { osr::OSRNewSpatialReference(c_str.as_ptr()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference").into());
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_epsg(epsg_code: u32) -> Result<SpatialRef> {
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { osr::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { osr::OSRImportFromEPSG(c_obj, epsg_code as c_int) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(rv, "OSRImportFromEPSG").into())
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn from_proj4(proj4_string: &str) -> Result<SpatialRef> {
        let c_str = CString::new(proj4_string)?;
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { osr::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { osr::OSRImportFromProj4(c_obj, c_str.as_ptr()) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(rv, "OSRImportFromProj4").into())
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn from_esri(esri_wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(esri_wkt)?;
        let ptrs = vec![c_str.as_ptr(), ptr::null_mut()];
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { osr::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { osr::OSRImportFromESRI(c_obj, ptrs.as_ptr()) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(rv, "OSRImportFromESRI").into())
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn from_c_obj(c_obj: *const c_void) -> Result<SpatialRef> {
        let mut_c_obj = unsafe { osr::OSRClone(c_obj) };
        if mut_c_obj.is_null() {
           return Err(_last_null_pointer_err("OSRClone").into());
        } else {
            Ok(SpatialRef(mut_c_obj))
        }
    }

    pub fn to_wkt(&self) -> Result<String> {
        let mut c_wkt: *const c_char = ptr::null_mut();
        let _err = unsafe { osr::OSRExportToWkt(self.0, &mut c_wkt) };
        if _err != ogr_enums::OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToWkt").into())
        } else {
            Ok(_string(c_wkt))
        }
    }

    pub fn morph_to_esri(&self) -> Result<()> {
        let _err = unsafe { osr::OSRMorphToESRI(self.0) };
        if _err != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(_err, "OSRMorphToESRI").into());
        }
        Ok(())
    }

    pub fn to_pretty_wkt(&self) -> Result<String> {
        let mut c_wkt: *const c_char = ptr::null_mut();
        let _err = unsafe { osr::OSRExportToPrettyWkt(self.0, &mut c_wkt, false as c_int) };
        if _err != ogr_enums::OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToPrettyWkt").into())
        } else {
            Ok(_string(c_wkt))
        }
    }

    pub fn to_xml(&self) -> Result<String> {
        let mut c_raw_xml: *const c_char = ptr::null_mut();
        let _err = unsafe { osr::OSRExportToXML(self.0, &mut c_raw_xml, ptr::null() as *const c_char) };
        if _err != ogr_enums::OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToXML").into())
        } else {
            Ok(_string(c_raw_xml))
        }
    }

    pub fn to_proj4(&self) -> Result<String> {
        let mut c_proj4str: *const c_char = ptr::null_mut();
        let _err = unsafe { osr::OSRExportToProj4(self.0, &mut c_proj4str) };
        if _err != ogr_enums::OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToProj4").into())
        } else {
            Ok(_string(c_proj4str))
        }
    }

    pub fn auth_name(&self) -> Result<String> {
        let c_ptr = unsafe { osr::OSRGetAuthorityName(self.0, ptr::null() as *const c_char) };
        if c_ptr.is_null() {
            Err(_last_null_pointer_err("SRGetAuthorityName").into())
        } else {
            Ok(_string(c_ptr))
        }
    }

    pub fn auth_code(&self) -> Result<i32> {
        let c_ptr = unsafe { osr::OSRGetAuthorityCode(self.0, ptr::null() as *const c_char) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAuthorityCode").into());
        }
        let c_str = unsafe { CStr::from_ptr(c_ptr) };
        let epsg = i32::from_str(c_str.to_str()?);
        match epsg {
            Ok(n) => Ok(n),
            Err(_) => Err(ErrorKind::OgrError(OGRErr::OGRERR_UNSUPPORTED_SRS, "OSRGetAuthorityCode").into())
        }
    }

    pub fn authority(&self) -> Result<String> {
        let c_ptr = unsafe { osr::OSRGetAuthorityName(self.0, ptr::null() as *const c_char) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("SRGetAuthorityName").into());
        }
        let name = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        let c_ptr = unsafe { osr::OSRGetAuthorityCode(self.0, ptr::null() as *const c_char) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAuthorityCode").into());
        }
        let code = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        Ok(format!("{}:{}", name, code))
    }

    pub fn to_c_hsrs(&self) -> *const c_void {
        self.0 as *const c_void
    }
}
