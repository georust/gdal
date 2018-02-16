use libc::c_int;
use std::ffi::{CString, CStr};
use std::ptr;
use std::str::FromStr;
use utils::{_string, _last_null_pointer_err, _last_cpl_err};
use gdal_sys::{self, CPLErr, OGRCoordinateTransformationH, OGRErr, OGRSpatialReferenceH, TRUE};

use errors::*;

pub struct CoordTransform{
    inner: OGRCoordinateTransformationH,
    from: String,
    to: String,
}

impl Drop for CoordTransform {
    fn drop(&mut self) {
        unsafe { gdal_sys::OCTDestroyCoordinateTransformation(self.inner) };
        self.inner = ptr::null_mut();
    }
}

impl CoordTransform {
    pub fn new(sp_ref1: &SpatialRef, sp_ref2: &SpatialRef) -> Result<CoordTransform> {
        let c_obj = unsafe { gdal_sys::OCTNewCoordinateTransformation(sp_ref1.0, sp_ref2.0) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OCTNewCoordinateTransformation").into());
        }
        Ok(CoordTransform{
            inner: c_obj,
            from: sp_ref1.authority().or_else(|_| sp_ref1.to_proj4())?,
            to: sp_ref2.authority().or_else(|_| sp_ref2.to_proj4())?
        })
    }

    pub fn transform_coords(&self, x: &mut [f64], y: &mut [f64], z: &mut [f64]) -> Result<()> {
        let nb_coords = x.len();
        assert_eq!(nb_coords, y.len());
        let ret_val = unsafe {
            gdal_sys::OCTTransform(
                self.inner,
                nb_coords as c_int,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                z.as_mut_ptr(),
            ) == TRUE as i32
        };

        if ret_val {
            Ok(())
        } else {
            let err = _last_cpl_err(CPLErr::CE_Failure);
            let msg = if let ErrorKind::CplError(_, _, msg) = err {
                if msg.trim().len() == 0 {
                    None
                } else {
                    Some(msg)
                }
            } else {
                return Err(err.into());
            };
            Err(ErrorKind::InvalidCoordinateRange(self.from.clone(), self.to.clone(), msg).into())
        }
    }

    #[deprecated(since = "0.3.1", note = "use `transform_coords` instead")]
    pub fn transform_coord(&self, x: &mut [f64], y: &mut [f64], z: &mut [f64]){
        self.transform_coords(x, y, z).expect("Coordinate transform successful")
    }

    pub fn to_c_hct(&self) -> OGRCoordinateTransformationH {
        self.inner
    }
}

#[derive(Debug)]
pub struct SpatialRef(OGRSpatialReferenceH);

impl Drop for SpatialRef {
    fn drop(&mut self){
        unsafe { gdal_sys::OSRRelease(self.0)};
        self.0 = ptr::null_mut();
    }
}

impl Clone for SpatialRef {
    fn clone(&self) -> SpatialRef {
        let n_obj = unsafe { gdal_sys::OSRClone(self.0)};
        SpatialRef(n_obj)
    }
}

impl PartialEq for SpatialRef {
    fn eq(&self, other: &SpatialRef) -> bool {
        unsafe { gdal_sys::OSRIsSame(self.0, other.0) == TRUE as i32 }
    }
}

impl SpatialRef {
    pub fn new() -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(ptr::null()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference").into());
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_definition(definition: &str) -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(ptr::null()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference").into());
        }
        let rv = unsafe { gdal_sys::OSRSetFromUserInput(c_obj, CString::new(definition)?.as_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OSRSetFromUserInput").into());
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_wkt(wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(wkt)?;
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(c_str.as_ptr()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference").into());
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_epsg(epsg_code: u32) -> Result<SpatialRef> {
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { gdal_sys::OSRImportFromEPSG(c_obj, epsg_code as c_int) };
        if rv != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(rv, "OSRImportFromEPSG").into())
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn from_proj4(proj4_string: &str) -> Result<SpatialRef> {
        let c_str = CString::new(proj4_string)?;
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { gdal_sys::OSRImportFromProj4(c_obj, c_str.as_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(rv, "OSRImportFromProj4").into())
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn from_esri(esri_wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(esri_wkt)?;
        let mut ptrs = vec![c_str.as_ptr() as *mut i8, ptr::null_mut()];
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { gdal_sys::OSRImportFromESRI(c_obj, ptrs.as_mut_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(rv, "OSRImportFromESRI").into())
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn from_c_obj(c_obj: OGRSpatialReferenceH) -> Result<SpatialRef> {
        let mut_c_obj = unsafe { gdal_sys::OSRClone(c_obj) };
        if mut_c_obj.is_null() {
           return Err(_last_null_pointer_err("OSRClone").into());
        } else {
            Ok(SpatialRef(mut_c_obj))
        }
    }

    pub fn to_wkt(&self) -> Result<String> {
        let mut c_wkt = ptr::null_mut();
        let _err = unsafe { gdal_sys::OSRExportToWkt(self.0, &mut c_wkt) };
        if _err != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToWkt").into())
        } else {
            Ok(_string(c_wkt))
        }
    }

    pub fn morph_to_esri(&self) -> Result<()> {
        let _err = unsafe { gdal_sys::OSRMorphToESRI(self.0) };
        if _err != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(_err, "OSRMorphToESRI").into());
        }
        Ok(())
    }

    pub fn to_pretty_wkt(&self) -> Result<String> {
        let mut c_wkt = ptr::null_mut();
        let _err = unsafe { gdal_sys::OSRExportToPrettyWkt(self.0, &mut c_wkt, false as c_int) };
        if _err != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToPrettyWkt").into())
        } else {
            Ok(_string(c_wkt))
        }
    }

    pub fn to_xml(&self) -> Result<String> {
        let mut c_raw_xml = ptr::null_mut();
        let _err = unsafe { gdal_sys::OSRExportToXML(self.0, &mut c_raw_xml, ptr::null()) };
        if _err != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToXML").into())
        } else {
            Ok(_string(c_raw_xml))
        }
    }

    pub fn to_proj4(&self) -> Result<String> {
        let mut c_proj4str = ptr::null_mut();
        let _err = unsafe { gdal_sys::OSRExportToProj4(self.0, &mut c_proj4str) };
        if _err != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRExportToProj4").into())
        } else {
            Ok(_string(c_proj4str))
        }
    }

    pub fn auth_name(&self) -> Result<String> {
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityName(self.0, ptr::null()) };
        if c_ptr.is_null() {
            Err(_last_null_pointer_err("SRGetAuthorityName").into())
        } else {
            Ok(_string(c_ptr))
        }
    }

    pub fn auth_code(&self) -> Result<i32> {
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityCode(self.0, ptr::null()) };
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
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityName(self.0, ptr::null()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("SRGetAuthorityName").into());
        }
        let name = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityCode(self.0, ptr::null()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAuthorityCode").into());
        }
        let code = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        Ok(format!("{}:{}", name, code))
    }

    pub fn auto_identify_epsg(&mut self) -> Result<()> {
        let _err = unsafe { gdal_sys::OSRAutoIdentifyEPSG(self.0) };
        if _err != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError(_err, "OSRAutoIdentifyEPSG").into())
        } else {
            Ok(())
        }
    }

    pub fn to_c_hsrs(&self) -> OGRSpatialReferenceH {
        self.0
    }
}
