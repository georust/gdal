use crate::utils::{_last_null_pointer_err, _string};
use gdal_sys::{self, OGRErr};
use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::ptr::{self};
use std::str::FromStr;

use crate::errors::*;

#[derive(Debug, Clone)]
pub struct AreaOfUse {
    pub west_lon_degree: f64,
    pub south_lat_degree: f64,
    pub east_lon_degree: f64,
    pub north_lat_degree: f64,
    pub name: String,
}

pub type AxisOrientationType = gdal_sys::OGRAxisOrientation::Type;

#[derive(Debug)]
pub struct SpatialRef(gdal_sys::OGRSpatialReferenceH);

impl Drop for SpatialRef {
    fn drop(&mut self) {
        unsafe { gdal_sys::OSRRelease(self.0) };
        self.0 = ptr::null_mut();
    }
}

impl Clone for SpatialRef {
    fn clone(&self) -> SpatialRef {
        let n_obj = unsafe { gdal_sys::OSRClone(self.0) };
        SpatialRef(n_obj)
    }
}

impl PartialEq for SpatialRef {
    fn eq(&self, other: &SpatialRef) -> bool {
        unsafe { gdal_sys::OSRIsSame(self.0, other.0) == 1 }
    }
}

impl SpatialRef {
    pub fn new() -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(ptr::null()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference"));
        }
        Ok(SpatialRef(c_obj))
    }

    /// Returns a wrapped `SpatialRef` from a raw C API handle.
    ///
    /// # Safety
    /// The handle passed to this function must be valid.
    pub unsafe fn from_c_obj(c_obj: gdal_sys::OGRSpatialReferenceH) -> Result<SpatialRef> {
        let mut_c_obj = gdal_sys::OSRClone(c_obj);
        if mut_c_obj.is_null() {
            Err(_last_null_pointer_err("OSRClone"))
        } else {
            Ok(SpatialRef(mut_c_obj))
        }
    }

    /// Get the handle to underlying C API data
    pub(crate) unsafe fn c_handle(&self) -> gdal_sys::OGRSpatialReferenceH {
        self.0
    }

    pub fn from_definition(definition: &str) -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(ptr::null()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference"));
        }
        let rv =
            unsafe { gdal_sys::OSRSetFromUserInput(c_obj, CString::new(definition)?.as_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRSetFromUserInput",
            });
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_wkt(wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(wkt)?;
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(c_str.as_ptr()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference"));
        }
        Ok(SpatialRef(c_obj))
    }

    pub fn from_epsg(epsg_code: u32) -> Result<SpatialRef> {
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { gdal_sys::OSRImportFromEPSG(c_obj, epsg_code as c_int) };
        if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRImportFromEPSG",
            })
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
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRImportFromProj4",
            })
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn from_esri(esri_wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(esri_wkt)?;
        let mut ptrs = vec![c_str.as_ptr() as *mut c_char, ptr::null_mut()];
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { gdal_sys::OSRImportFromESRI(c_obj, ptrs.as_mut_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRImportFromESRI",
            })
        } else {
            Ok(SpatialRef(c_obj))
        }
    }

    pub fn to_wkt(&self) -> Result<String> {
        let mut c_wkt = ptr::null_mut();
        let rv = unsafe { gdal_sys::OSRExportToWkt(self.0, &mut c_wkt) };
        let res = if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRExportToWkt",
            })
        } else {
            Ok(_string(c_wkt))
        };
        unsafe { gdal_sys::VSIFree(c_wkt.cast::<std::ffi::c_void>()) };
        res
    }

    pub fn morph_to_esri(&self) -> Result<()> {
        let rv = unsafe { gdal_sys::OSRMorphToESRI(self.0) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRMorphToESRI",
            });
        }
        Ok(())
    }

    pub fn to_pretty_wkt(&self) -> Result<String> {
        let mut c_wkt = ptr::null_mut();
        let rv = unsafe { gdal_sys::OSRExportToPrettyWkt(self.0, &mut c_wkt, false as c_int) };
        let res = if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRExportToPrettyWkt",
            })
        } else {
            Ok(_string(c_wkt))
        };
        unsafe { gdal_sys::VSIFree(c_wkt.cast::<std::ffi::c_void>()) };
        res
    }

    pub fn to_xml(&self) -> Result<String> {
        let mut c_raw_xml = ptr::null_mut();
        let rv = unsafe { gdal_sys::OSRExportToXML(self.0, &mut c_raw_xml, ptr::null()) };
        let res = if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRExportToXML",
            })
        } else {
            Ok(_string(c_raw_xml))
        };
        unsafe { gdal_sys::VSIFree(c_raw_xml.cast::<std::ffi::c_void>()) };
        res
    }

    pub fn to_proj4(&self) -> Result<String> {
        let mut c_proj4str = ptr::null_mut();
        let rv = unsafe { gdal_sys::OSRExportToProj4(self.0, &mut c_proj4str) };
        let res = if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRExportToProj4",
            })
        } else {
            Ok(_string(c_proj4str))
        };
        unsafe { gdal_sys::VSIFree(c_proj4str.cast::<std::ffi::c_void>()) };
        res
    }

    #[cfg(any(major_ge_4, all(major_ge_3, minor_ge_1)))]
    pub fn to_projjson(&self) -> Result<String> {
        let mut c_projjsonstr = ptr::null_mut();
        let options = ptr::null();
        let rv = unsafe { gdal_sys::OSRExportToPROJJSON(self.0, &mut c_projjsonstr, options) };
        let res = if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRExportToPROJJSON",
            })
        } else {
            Ok(_string(c_projjsonstr))
        };
        unsafe { gdal_sys::VSIFree(c_projjsonstr.cast::<std::ffi::c_void>()) };
        res
    }

    pub fn auth_name(&self) -> Result<String> {
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityName(self.0, ptr::null()) };
        if c_ptr.is_null() {
            Err(_last_null_pointer_err("SRGetAuthorityName"))
        } else {
            Ok(_string(c_ptr))
        }
    }

    pub fn auth_code(&self) -> Result<i32> {
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityCode(self.0, ptr::null()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAuthorityCode"));
        }
        let c_str = unsafe { CStr::from_ptr(c_ptr) };
        let epsg = i32::from_str(c_str.to_str()?);
        match epsg {
            Ok(n) => Ok(n),
            Err(_) => Err(GdalError::OgrError {
                err: OGRErr::OGRERR_UNSUPPORTED_SRS,
                method_name: "OSRGetAuthorityCode",
            }),
        }
    }

    pub fn authority(&self) -> Result<String> {
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityName(self.0, ptr::null()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("SRGetAuthorityName"));
        }
        let name = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityCode(self.0, ptr::null()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAuthorityCode"));
        }
        let code = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        Ok(format!("{name}:{code}"))
    }

    pub fn auto_identify_epsg(&mut self) -> Result<()> {
        let rv = unsafe { gdal_sys::OSRAutoIdentifyEPSG(self.0) };
        if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRAutoIdentifyEPSG",
            })
        } else {
            Ok(())
        }
    }

    #[cfg(major_ge_3)]
    pub fn name(&self) -> Result<String> {
        let c_ptr = unsafe { gdal_sys::OSRGetName(self.0) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetName"));
        }
        Ok(_string(c_ptr))
    }

    pub fn angular_units_name(&self) -> Result<String> {
        let mut c_ptr = ptr::null_mut();
        unsafe { gdal_sys::OSRGetAngularUnits(self.0, &mut c_ptr) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAngularUnits"));
        }
        Ok(_string(c_ptr))
    }

    pub fn angular_units(&self) -> f64 {
        unsafe { gdal_sys::OSRGetAngularUnits(self.0, ptr::null_mut()) }
    }

    pub fn linear_units_name(&self) -> Result<String> {
        let mut c_ptr = ptr::null_mut();
        unsafe { gdal_sys::OSRGetLinearUnits(self.0, &mut c_ptr) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetLinearUnits"));
        }
        Ok(_string(c_ptr))
    }

    pub fn linear_units(&self) -> f64 {
        unsafe { gdal_sys::OSRGetLinearUnits(self.0, ptr::null_mut()) }
    }

    #[inline]
    pub fn is_geographic(&self) -> bool {
        unsafe { gdal_sys::OSRIsGeographic(self.0) == 1 }
    }

    #[inline]
    #[cfg(all(major_ge_3, minor_ge_1))]
    pub fn is_derived_geographic(&self) -> bool {
        unsafe { gdal_sys::OSRIsDerivedGeographic(self.0) == 1 }
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        unsafe { gdal_sys::OSRIsLocal(self.0) == 1 }
    }

    #[inline]
    pub fn is_projected(&self) -> bool {
        unsafe { gdal_sys::OSRIsProjected(self.0) == 1 }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        unsafe { gdal_sys::OSRIsCompound(self.0) == 1 }
    }

    #[inline]
    pub fn is_geocentric(&self) -> bool {
        unsafe { gdal_sys::OSRIsGeocentric(self.0) == 1 }
    }

    #[inline]
    pub fn is_vertical(&self) -> bool {
        unsafe { gdal_sys::OSRIsVertical(self.0) == 1 }
    }

    pub fn axis_orientation(&self, target_key: &str, axis: i32) -> Result<AxisOrientationType> {
        let mut orientation = gdal_sys::OGRAxisOrientation::OAO_Other;
        let c_ptr = unsafe {
            gdal_sys::OSRGetAxis(
                self.0,
                CString::new(target_key)?.as_ptr(),
                axis as c_int,
                &mut orientation,
            )
        };
        // null ptr indicate a failure (but no CPLError) see Gdal documentation.
        if c_ptr.is_null() {
            Err(GdalError::AxisNotFoundError {
                key: target_key.into(),
                method_name: "OSRGetAxis",
            })
        } else {
            Ok(orientation)
        }
    }

    pub fn axis_name(&self, target_key: &str, axis: i32) -> Result<String> {
        // See get_axis_orientation
        let c_ptr = unsafe {
            gdal_sys::OSRGetAxis(
                self.0,
                CString::new(target_key)?.as_ptr(),
                axis as c_int,
                ptr::null_mut(),
            )
        };
        if c_ptr.is_null() {
            Err(GdalError::AxisNotFoundError {
                key: target_key.into(),
                method_name: "OSRGetAxis",
            })
        } else {
            Ok(_string(c_ptr))
        }
    }

    #[cfg(all(major_ge_3, minor_ge_1))]
    pub fn axes_count(&self) -> i32 {
        unsafe { gdal_sys::OSRGetAxesCount(self.0) }
    }

    #[cfg(major_ge_3)]
    pub fn set_axis_mapping_strategy(&self, strategy: gdal_sys::OSRAxisMappingStrategy::Type) {
        unsafe {
            gdal_sys::OSRSetAxisMappingStrategy(self.0, strategy);
        }
    }

    #[cfg(major_ge_3)]
    #[deprecated(note = "use `axis_mapping_strategy` instead")]
    pub fn get_axis_mapping_strategy(&self) -> gdal_sys::OSRAxisMappingStrategy::Type {
        self.axis_mapping_strategy()
    }

    #[cfg(major_ge_3)]
    pub fn axis_mapping_strategy(&self) -> gdal_sys::OSRAxisMappingStrategy::Type {
        unsafe { gdal_sys::OSRGetAxisMappingStrategy(self.0) }
    }

    #[cfg(major_ge_3)]
    pub fn area_of_use(&self) -> Option<AreaOfUse> {
        let mut c_area_name: *const libc::c_char = ptr::null_mut();
        let (mut w_long, mut s_lat, mut e_long, mut n_lat): (f64, f64, f64, f64) =
            (0.0, 0.0, 0.0, 0.0);
        let ret_val = unsafe {
            gdal_sys::OSRGetAreaOfUse(
                self.0,
                &mut w_long,
                &mut s_lat,
                &mut e_long,
                &mut n_lat,
                &mut c_area_name,
            ) == 1
        };

        if ret_val {
            Some(AreaOfUse {
                west_lon_degree: w_long,
                south_lat_degree: s_lat,
                east_lon_degree: e_long,
                north_lat_degree: n_lat,
                name: _string(c_area_name),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_almost_eq;

    #[test]
    fn from_wkt_to_proj4() {
        let spatial_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
        assert_eq!(
            "+proj=longlat +ellps=WGS84 +towgs84=0,0,0,0,0,0,0 +no_defs",
            spatial_ref.to_proj4().unwrap().trim()
        );
        let spatial_ref = SpatialRef::from_definition("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
        assert_eq!(
            "+proj=longlat +ellps=WGS84 +towgs84=0,0,0,0,0,0,0 +no_defs",
            spatial_ref.to_proj4().unwrap().trim()
        );
    }

    #[test]
    fn from_proj4_to_wkt() {
        let spatial_ref = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
        // TODO: handle proj changes on lib level
        #[cfg(not(major_ge_3))]
        assert_eq!(spatial_ref.to_wkt().unwrap(), "PROJCS[\"unnamed\",GEOGCS[\"GRS 1980(IUGG, 1980)\",DATUM[\"unknown\",SPHEROID[\"GRS80\",6378137,298.257222101]],PRIMEM[\"Greenwich\",0],UNIT[\"degree\",0.0174532925199433]],PROJECTION[\"Lambert_Azimuthal_Equal_Area\"],PARAMETER[\"latitude_of_center\",52],PARAMETER[\"longitude_of_center\",10],PARAMETER[\"false_easting\",4321000],PARAMETER[\"false_northing\",3210000],UNIT[\"Meter\",1]]");
        #[cfg(major_ge_3)]
        assert_eq!(spatial_ref.to_wkt().unwrap(), "PROJCS[\"unknown\",GEOGCS[\"unknown\",DATUM[\"Unknown based on GRS80 ellipsoid\",SPHEROID[\"GRS 1980\",6378137,298.257222101,AUTHORITY[\"EPSG\",\"7019\"]]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]]],PROJECTION[\"Lambert_Azimuthal_Equal_Area\"],PARAMETER[\"latitude_of_center\",52],PARAMETER[\"longitude_of_center\",10],PARAMETER[\"false_easting\",4321000],PARAMETER[\"false_northing\",3210000],UNIT[\"metre\",1,AUTHORITY[\"EPSG\",\"9001\"]],AXIS[\"Easting\",EAST],AXIS[\"Northing\",NORTH]]");
    }

    #[test]
    fn from_epsg_to_wkt_proj4() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
        let wkt = spatial_ref.to_wkt().unwrap();
        // TODO: handle proj changes on lib level
        #[cfg(not(major_ge_3))]
        assert_eq!("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",\"7030\"]],AUTHORITY[\"EPSG\",\"6326\"]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]],AUTHORITY[\"EPSG\",\"4326\"]]", wkt);
        #[cfg(major_ge_3)]
        assert_eq!("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",\"7030\"]],AUTHORITY[\"EPSG\",\"6326\"]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]],AXIS[\"Latitude\",NORTH],AXIS[\"Longitude\",EAST],AUTHORITY[\"EPSG\",\"4326\"]]", wkt);
        let proj4string = spatial_ref.to_proj4().unwrap();
        assert_eq!("+proj=longlat +datum=WGS84 +no_defs", proj4string.trim());
    }

    #[cfg(any(major_ge_4, all(major_ge_3, minor_ge_1)))]
    #[test]
    fn from_epsg_to_projjson() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
        let projjson = spatial_ref.to_projjson().unwrap();
        // Testing for exact string equality would be too strict, since the order of keys in JSON is
        // unspecified. Ideally, we'd parse the JSON and then compare the values, but adding a JSON
        // parser as a dependency just for this one test would be overkill. Thus, we do only a quick
        // sanity check.
        assert!(
            projjson.contains("World Geodetic System 1984"),
            "{projjson:?} does not contain expected CRS name",
        );
    }

    #[test]
    fn from_esri_to_proj4() {
        let spatial_ref = SpatialRef::from_esri("GEOGCS[\"GCS_WGS_1984\",DATUM[\"D_WGS_1984\",SPHEROID[\"WGS_1984\",6378137,298.257223563]],PRIMEM[\"Greenwich\",0],UNIT[\"Degree\",0.017453292519943295]]").unwrap();
        let proj4string = spatial_ref.to_proj4().unwrap();
        assert_eq!("+proj=longlat +datum=WGS84 +no_defs", proj4string.trim());
    }

    #[test]
    fn comparison() {
        let spatial_ref1 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
        let spatial_ref2 = SpatialRef::from_epsg(4326).unwrap();
        let spatial_ref3 = SpatialRef::from_epsg(3025).unwrap();
        let spatial_ref4 = SpatialRef::from_proj4("+proj=longlat +datum=WGS84 +no_defs ").unwrap();
        let spatial_ref5 = SpatialRef::from_esri("GEOGCS[\"GCS_WGS_1984\",DATUM[\"D_WGS_1984\",SPHEROID[\"WGS_1984\",6378137,298.257223563]],PRIMEM[\"Greenwich\",0],UNIT[\"Degree\",0.017453292519943295]]").unwrap();

        assert_eq!(spatial_ref1, spatial_ref2);
        assert_ne!(spatial_ref2, spatial_ref3);
        assert_eq!(spatial_ref4, spatial_ref2);
        assert_eq!(spatial_ref5, spatial_ref4);
    }

    #[test]
    fn authority() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
        assert_eq!(spatial_ref.auth_name().unwrap(), "EPSG".to_string());
        assert_eq!(spatial_ref.auth_code().unwrap(), 4326);
        assert_eq!(spatial_ref.authority().unwrap(), "EPSG:4326".to_string());
        let spatial_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
        assert_eq!(spatial_ref.auth_name().unwrap(), "EPSG".to_string());
        assert_eq!(spatial_ref.auth_code().unwrap(), 4326);
        assert_eq!(spatial_ref.authority().unwrap(), "EPSG:4326".to_string());
        let spatial_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST]]").unwrap();
        assert!(spatial_ref.auth_name().is_err());
        assert!(spatial_ref.auth_code().is_err());
        assert!(spatial_ref.authority().is_err());
        let spatial_ref = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
        assert!(spatial_ref.auth_name().is_err());
        assert!(spatial_ref.auth_code().is_err());
        assert!(spatial_ref.authority().is_err());
    }

    #[test]
    fn auto_identify() {
        // retreived from https://epsg.io/32632, but deleted the `AUTHORITY["EPSG","32632"]`
        let mut spatial_ref = SpatialRef::from_wkt(
            r#"
        PROJCS["WGS 84 / UTM zone 32N",
            GEOGCS["WGS 84",
                DATUM["WGS_1984",
                    SPHEROID["WGS 84",6378137,298.257223563,
                        AUTHORITY["EPSG","7030"]],
                    AUTHORITY["EPSG","6326"]],
                PRIMEM["Greenwich",0,
                    AUTHORITY["EPSG","8901"]],
                UNIT["degree",0.0174532925199433,
                    AUTHORITY["EPSG","9122"]],
                AUTHORITY["EPSG","4326"]],
            PROJECTION["Transverse_Mercator"],
            PARAMETER["latitude_of_origin",0],
            PARAMETER["central_meridian",9],
            PARAMETER["scale_factor",0.9996],
            PARAMETER["false_easting",500000],
            PARAMETER["false_northing",0],
            UNIT["metre",1,
                AUTHORITY["EPSG","9001"]],
            AXIS["Easting",EAST],
            AXIS["Northing",NORTH]]
    "#,
        )
        .unwrap();
        assert!(spatial_ref.auth_code().is_err());
        spatial_ref.auto_identify_epsg().unwrap();
        assert_eq!(spatial_ref.auth_code().unwrap(), 32632);
    }

    #[cfg(major_ge_3)]
    #[test]
    fn axis_mapping_strategy() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
        assert_eq!(
            spatial_ref.axis_mapping_strategy(),
            gdal_sys::OSRAxisMappingStrategy::OAMS_AUTHORITY_COMPLIANT
        );
        spatial_ref.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        assert_eq!(
            spatial_ref.axis_mapping_strategy(),
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER
        );
    }

    #[cfg(major_ge_3)]
    #[test]
    fn area_of_use() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
        let area_of_use = spatial_ref.area_of_use().unwrap();
        assert_almost_eq(area_of_use.west_lon_degree, -180.0);
        assert_almost_eq(area_of_use.south_lat_degree, -90.0);
        assert_almost_eq(area_of_use.east_lon_degree, 180.0);
        assert_almost_eq(area_of_use.north_lat_degree, 90.0);
    }

    #[cfg(major_ge_3)]
    #[test]
    fn get_name() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
        let name = spatial_ref.name().unwrap();
        assert_eq!(name, "WGS 84");
    }

    #[test]
    fn get_units_epsg4326() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();

        let angular_units_name = spatial_ref.angular_units_name().unwrap();
        assert_eq!(angular_units_name.to_lowercase(), "degree");
        let to_radians = spatial_ref.angular_units();
        assert_almost_eq(to_radians, 0.01745329);
    }

    #[test]
    fn get_units_epsg2154() {
        let spatial_ref = SpatialRef::from_epsg(2154).unwrap();
        let linear_units_name = spatial_ref.linear_units_name().unwrap();
        assert_eq!(linear_units_name.to_lowercase(), "metre");
        let to_meters = spatial_ref.linear_units();
        assert_almost_eq(to_meters, 1.0);
    }

    #[test]
    fn predicats_epsg4326() {
        let spatial_ref_4326 = SpatialRef::from_epsg(4326).unwrap();
        assert!(spatial_ref_4326.is_geographic());
        assert!(!spatial_ref_4326.is_local());
        assert!(!spatial_ref_4326.is_projected());
        assert!(!spatial_ref_4326.is_compound());
        assert!(!spatial_ref_4326.is_geocentric());
        assert!(!spatial_ref_4326.is_vertical());

        #[cfg(all(major_ge_3, minor_ge_1))]
        assert!(!spatial_ref_4326.is_derived_geographic());
    }

    #[test]
    fn predicats_epsg2154() {
        let spatial_ref_2154 = SpatialRef::from_epsg(2154).unwrap();
        assert!(!spatial_ref_2154.is_geographic());
        assert!(!spatial_ref_2154.is_local());
        assert!(spatial_ref_2154.is_projected());
        assert!(!spatial_ref_2154.is_compound());
        assert!(!spatial_ref_2154.is_geocentric());

        #[cfg(all(major_ge_3, minor_ge_1))]
        assert!(!spatial_ref_2154.is_derived_geographic());
    }

    //XXX Gdal 2 implementation is partial
    #[cfg(major_ge_3)]
    #[test]
    fn crs_axis() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();

        #[cfg(all(major_ge_3, minor_ge_1))]
        assert_eq!(spatial_ref.axes_count(), 2);

        let orientation = spatial_ref.axis_orientation("GEOGCS", 0).unwrap();
        assert_eq!(orientation, gdal_sys::OGRAxisOrientation::OAO_North);
        assert!(spatial_ref.axis_name("GEOGCS", 0).is_ok());
        assert!(spatial_ref.axis_name("DO_NO_EXISTS", 0).is_err());
        assert!(spatial_ref.axis_orientation("DO_NO_EXISTS", 0).is_err());
    }
}
