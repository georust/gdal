use crate::utils::{_last_null_pointer_err, _string};
use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};
use gdal_sys::{self, OGRErr};
use std::ffi::{CStr, CString};
use std::ptr::{self};
use std::str::FromStr;

use crate::errors::*;

foreign_type! {
    /// A OpenGIS Spatial Reference System definition.
    ///
    /// Used in geo-referencing raster and vector data, and in coordinate transformations.
    ///
    /// # Notes
    /// * See also: [OGR Coordinate Reference Systems and Coordinate Transformation Tutorial](https://gdal.org/tutorials/osr_api_tut.html)
    /// * Consult the [OGC WKT Coordinate System Issues](https://gdal.org/tutorials/wktproblems.html)
    /// page for implementation details of WKT in OGR.
    #[derive(Debug)]
    pub unsafe type SpatialRef {
        type CType = libc::c_void;
        fn drop = gdal_sys::OSRRelease;
        fn clone = gdal_sys::OSRClone;
    }
}

impl PartialEq for SpatialRef {
    fn eq(&self, other: &SpatialRef) -> bool {
        unsafe { gdal_sys::OSRIsSame(self.as_ptr(), other.as_ptr()) == 1 }
    }
}

impl SpatialRef {
    pub fn new() -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(ptr::null()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference"));
        }
        Ok(unsafe { Self::from_ptr(c_obj) })
    }

    /// Set spatial reference from various text formats.
    ///
    /// This method will examine the provided input, and try to deduce the format,
    /// and then use it to initialize the spatial reference system. See the [C++ API docs][CPP]
    /// for details on these forms.
    ///
    /// [CPP]: https://gdal.org/api/ogrspatialref.html#_CPPv4N19OGRSpatialReference16SetFromUserInputEPKc
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
        Ok(unsafe { Self::from_ptr(c_obj) })
    }

    pub fn from_wkt(wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(wkt)?;
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(c_str.as_ptr()) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OSRNewSpatialReference"));
        }
        Ok(unsafe { Self::from_ptr(c_obj) })
    }

    pub fn from_epsg(epsg_code: u32) -> Result<SpatialRef> {
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { gdal_sys::OSRImportFromEPSG(c_obj, epsg_code as libc::c_int) };
        if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRImportFromEPSG",
            })
        } else {
            Ok(unsafe { Self::from_ptr(c_obj) })
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
            Ok(unsafe { Self::from_ptr(c_obj) })
        }
    }

    pub fn from_esri(esri_wkt: &str) -> Result<SpatialRef> {
        let c_str = CString::new(esri_wkt)?;
        let mut ptrs = vec![c_str.as_ptr() as *mut libc::c_char, ptr::null_mut()];
        let null_ptr = ptr::null_mut();
        let c_obj = unsafe { gdal_sys::OSRNewSpatialReference(null_ptr) };
        let rv = unsafe { gdal_sys::OSRImportFromESRI(c_obj, ptrs.as_mut_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRImportFromESRI",
            })
        } else {
            Ok(unsafe { Self::from_ptr(c_obj) })
        }
    }
}

impl SpatialRefRef {
    pub fn to_wkt(&self) -> Result<String> {
        let mut c_wkt = ptr::null_mut();
        let rv = unsafe { gdal_sys::OSRExportToWkt(self.as_ptr(), &mut c_wkt) };
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
        let rv = unsafe { gdal_sys::OSRMorphToESRI(self.as_ptr()) };
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
        let rv = unsafe {
            gdal_sys::OSRExportToPrettyWkt(self.as_ptr(), &mut c_wkt, false as libc::c_int)
        };
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
        let rv = unsafe { gdal_sys::OSRExportToXML(self.as_ptr(), &mut c_raw_xml, ptr::null()) };
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
        let rv = unsafe { gdal_sys::OSRExportToProj4(self.as_ptr(), &mut c_proj4str) };
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
        let rv =
            unsafe { gdal_sys::OSRExportToPROJJSON(self.as_ptr(), &mut c_projjsonstr, options) };
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
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityName(self.as_ptr(), ptr::null()) };
        if c_ptr.is_null() {
            Err(_last_null_pointer_err("SRGetAuthorityName"))
        } else {
            Ok(_string(c_ptr))
        }
    }

    pub fn auth_code(&self) -> Result<i32> {
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityCode(self.as_ptr(), ptr::null()) };
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
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityName(self.as_ptr(), ptr::null()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("SRGetAuthorityName"));
        }
        let name = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        let c_ptr = unsafe { gdal_sys::OSRGetAuthorityCode(self.as_ptr(), ptr::null()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAuthorityCode"));
        }
        let code = unsafe { CStr::from_ptr(c_ptr) }.to_str()?;
        Ok(format!("{name}:{code}"))
    }

    pub fn auto_identify_epsg(&mut self) -> Result<()> {
        let rv = unsafe { gdal_sys::OSRAutoIdentifyEPSG(self.as_ptr()) };
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
        let c_ptr = unsafe { gdal_sys::OSRGetName(self.as_ptr()) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetName"));
        }
        Ok(_string(c_ptr))
    }

    pub fn angular_units_name(&self) -> Result<String> {
        let mut c_ptr = ptr::null_mut();
        unsafe { gdal_sys::OSRGetAngularUnits(self.as_ptr(), &mut c_ptr) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetAngularUnits"));
        }
        Ok(_string(c_ptr))
    }

    pub fn angular_units(&self) -> f64 {
        unsafe { gdal_sys::OSRGetAngularUnits(self.as_ptr(), ptr::null_mut()) }
    }

    pub fn linear_units_name(&self) -> Result<String> {
        let mut c_ptr = ptr::null_mut();
        unsafe { gdal_sys::OSRGetLinearUnits(self.as_ptr(), &mut c_ptr) };
        if c_ptr.is_null() {
            return Err(_last_null_pointer_err("OSRGetLinearUnits"));
        }
        Ok(_string(c_ptr))
    }

    pub fn linear_units(&self) -> f64 {
        unsafe { gdal_sys::OSRGetLinearUnits(self.as_ptr(), ptr::null_mut()) }
    }

    #[inline]
    pub fn is_geographic(&self) -> bool {
        unsafe { gdal_sys::OSRIsGeographic(self.as_ptr()) == 1 }
    }

    #[inline]
    #[cfg(all(major_ge_3, minor_ge_1))]
    pub fn is_derived_geographic(&self) -> bool {
        unsafe { gdal_sys::OSRIsDerivedGeographic(self.as_ptr()) == 1 }
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        unsafe { gdal_sys::OSRIsLocal(self.as_ptr()) == 1 }
    }

    #[inline]
    pub fn is_projected(&self) -> bool {
        unsafe { gdal_sys::OSRIsProjected(self.as_ptr()) == 1 }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        unsafe { gdal_sys::OSRIsCompound(self.as_ptr()) == 1 }
    }

    #[inline]
    pub fn is_geocentric(&self) -> bool {
        unsafe { gdal_sys::OSRIsGeocentric(self.as_ptr()) == 1 }
    }

    #[inline]
    pub fn is_vertical(&self) -> bool {
        unsafe { gdal_sys::OSRIsVertical(self.as_ptr()) == 1 }
    }

    pub fn axis_orientation(
        &self,
        target_key: &str,
        axis: i32,
    ) -> Result<super::AxisOrientationType> {
        let mut orientation = gdal_sys::OGRAxisOrientation::OAO_Other;
        let c_ptr = unsafe {
            gdal_sys::OSRGetAxis(
                self.as_ptr(),
                CString::new(target_key)?.as_ptr(),
                axis as libc::c_int,
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
                self.as_ptr(),
                CString::new(target_key)?.as_ptr(),
                axis as libc::c_int,
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
        unsafe { gdal_sys::OSRGetAxesCount(self.as_ptr()) }
    }

    #[cfg(major_ge_3)]
    pub fn set_axis_mapping_strategy(&self, strategy: gdal_sys::OSRAxisMappingStrategy::Type) {
        unsafe {
            gdal_sys::OSRSetAxisMappingStrategy(self.as_ptr(), strategy);
        }
    }

    #[cfg(major_ge_3)]
    #[deprecated(note = "use `axis_mapping_strategy` instead")]
    pub fn get_axis_mapping_strategy(&self) -> gdal_sys::OSRAxisMappingStrategy::Type {
        self.axis_mapping_strategy()
    }

    #[cfg(major_ge_3)]
    pub fn axis_mapping_strategy(&self) -> gdal_sys::OSRAxisMappingStrategy::Type {
        unsafe { gdal_sys::OSRGetAxisMappingStrategy(self.as_ptr()) }
    }

    #[cfg(major_ge_3)]
    /// Get the valid use bounding area for this `SpatialRef`.
    ///
    /// See: [`OSRGetAreaOfUse`](https://gdal.org/api/ogr_srs_api.html#_CPPv415OSRGetAreaOfUse20OGRSpatialReferenceHPdPdPdPdPPKc)
    pub fn area_of_use(&self) -> Option<AreaOfUse> {
        let mut c_area_name: *const libc::c_char = ptr::null_mut();
        let (mut w_long, mut s_lat, mut e_long, mut n_lat): (f64, f64, f64, f64) =
            (0.0, 0.0, 0.0, 0.0);
        let ret_val = unsafe {
            gdal_sys::OSRGetAreaOfUse(
                self.as_ptr(),
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

    /// Get spheroid semi-major axis.
    ///
    /// Returns an error if the semi-major axis can't be found
    ///
    /// See: [`OSRGetSemiMajor`](https://gdal.org/api/ogr_srs_api.html#_CPPv415OSRGetSemiMajor20OGRSpatialReferenceHP6OGRErr)
    pub fn semi_major(&self) -> Result<f64> {
        let mut err_code = OGRErr::OGRERR_NONE;
        let a = unsafe { gdal_sys::OSRGetSemiMajor(self.as_ptr(), &mut err_code as *mut u32) };
        if err_code != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: err_code,
                method_name: "OSRGetSemiMajor",
            });
        }
        Ok(a)
    }

    /// Get spheroid semi-minor axis.
    ///
    /// Returns an error if the semi-minor axis can't be found
    ///
    /// See: [`OSRGetSemiMinor`](https://gdal.org/api/ogr_srs_api.html#_CPPv415OSRGetSemiMinor20OGRSpatialReferenceHP6OGRErr)
    pub fn semi_minor(&self) -> Result<f64> {
        let mut err_code = OGRErr::OGRERR_NONE;
        let b = unsafe { gdal_sys::OSRGetSemiMinor(self.as_ptr(), &mut err_code as *mut u32) };
        if err_code != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: err_code,
                method_name: "OSRGetSemiMinor",
            });
        }
        Ok(b)
    }

    /// Set a projection parameter value.
    ///
    /// Returns an error if there the `PROJCS` node is missing.
    ///
    /// See: [`OSRSetProjParm`](https://gdal.org/api/ogr_srs_api.html#_CPPv414OSRSetProjParm20OGRSpatialReferenceHPKcd)
    pub fn set_proj_param(&mut self, name: &str, value: f64) -> Result<()> {
        let c_name = CString::new(name)?;
        let rv = unsafe { gdal_sys::OSRSetProjParm(self.as_ptr(), c_name.as_ptr(), value) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRSetProjParm",
            });
        }
        Ok(())
    }

    /// Fetch a projection parameter value.
    ///
    /// Returns:
    /// * `Ok(Some(value))` - if parameter is sucessfully found,
    /// * `Ok(None)` - if parameter is not found (C library will return [OGRERR_FAILURE](https://gdal.org/api/vector_c_api.html#c.OGRERR_FAILURE)),
    /// * `Err(_)` - if there is a string conversion error or on other GDAL OGR error.
    ///
    /// See: [`OSRGetProjParm`](https://gdal.org/api/ogr_srs_api.html#_CPPv414OSRGetProjParm20OGRSpatialReferenceHPKcdP6OGRErr)
    pub fn get_proj_param(&self, name: &str) -> Result<Option<f64>> {
        let c_name = CString::new(name)?;
        let mut err_code = OGRErr::OGRERR_NONE;
        let p = unsafe {
            gdal_sys::OSRGetProjParm(
                self.as_ptr(),
                c_name.as_ptr(),
                0.0,
                &mut err_code as *mut u32,
            )
        };
        if err_code != OGRErr::OGRERR_NONE {
            match err_code {
                OGRErr::OGRERR_FAILURE => Ok(None),
                other_err => Err(GdalError::OgrError {
                    err: other_err,
                    method_name: "OSRGetProjParm",
                }),
            }
        } else {
            Ok(Some(p))
        }
    }

    /// Set attribute value in spatial reference.
    ///
    /// Returns error in case if path was invalid or on GDAL OGR error.
    ///
    /// See: [`OSRSetAttrValue`](https://gdal.org/api/ogr_srs_api.html#_CPPv415OSRSetAttrValue20OGRSpatialReferenceHPKcPKc)
    pub fn set_attr_value(&mut self, node_path: &str, new_value: Option<&str>) -> Result<()> {
        let c_node_path = CString::new(node_path)?;
        let c_new_value = match new_value {
            Some(value) => Some(CString::new(value)?),
            None => None,
        };
        let value_ptr = c_new_value
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(ptr::null());
        let rv =
            unsafe { gdal_sys::OSRSetAttrValue(self.as_ptr(), c_node_path.as_ptr(), value_ptr) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OSRSetAttrValue",
            });
        }
        Ok(())
    }

    /// Fetch indicated attribute of named node.
    ///
    /// Returns:
    /// * `Ok(Some(value))` - if node and attribute are sucessfully found,
    /// * `Ok(None)` - if node or attribute are not found (C library will return `nullptr`) or attribute contains no value,
    /// * `Err(_)` - if there is a string conversion error.
    ///
    /// See: [`OSRGetProjParm`](https://gdal.org/api/ogr_srs_api.html#_CPPv415OSRGetAttrValue20OGRSpatialReferenceHPKci)
    ///
    /// # Panics
    ///
    /// Panics if `child` is greater than [`libc::c_int::MAX`].
    pub fn get_attr_value(&self, node_path: &str, child: usize) -> Result<Option<String>> {
        let child = child.try_into().expect("`child` must fit in `c_int`");

        let c_node_path = CString::new(node_path)?;
        let rv = unsafe { gdal_sys::OSRGetAttrValue(self.as_ptr(), c_node_path.as_ptr(), child) };
        if rv.is_null() {
            Ok(None)
        } else {
            Ok(Some(_string(rv)))
        }
    }

    /// Make a duplicate of the `GEOGCS` node of this [`SpatialRef`].
    ///
    /// Returns an error if the `GEOGCS` node is missing.
    ///
    /// See: [OSRCloneGeogCS](https://gdal.org/api/ogr_srs_api.html#_CPPv414OSRCloneGeogCS20OGRSpatialReferenceH)
    pub fn geog_cs(&self) -> Result<SpatialRef> {
        let raw_ret = unsafe { gdal_sys::OSRCloneGeogCS(self.as_ptr()) };
        if raw_ret.is_null() {
            return Err(_last_null_pointer_err("OSRCloneGeogCS"));
        }

        Ok(unsafe { SpatialRef::from_ptr(raw_ret) })
    }
}

#[derive(Debug, Clone)]
/// Defines the bounding area of valid use for a [`SpatialRef`].
///
/// See [`area_of_use`][SpatialRef::area_of_use].
pub struct AreaOfUse {
    pub west_lon_degree: f64,
    pub south_lat_degree: f64,
    pub east_lon_degree: f64,
    pub north_lat_degree: f64,
    pub name: String,
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
        let wkt = spatial_ref.to_wkt().unwrap();
        // Test we the DATUM description has expected tokens
        assert!(wkt.contains("GRS") && wkt.contains("80"));
        // Test parsing back in gets us functionally the same SRS
        let round_trip = SpatialRef::from_wkt(&wkt).unwrap();
        assert_eq!(spatial_ref, round_trip);
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

    #[test]
    fn semi_major_and_semi_minor() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();

        let semi_major = spatial_ref.semi_major().unwrap();
        assert_almost_eq(semi_major, 6_378_137.0);

        let semi_minor = spatial_ref.semi_minor().unwrap();
        assert_almost_eq(semi_minor, 6_356_752.31);
    }

    #[test]
    fn proj_params() {
        let spatial_ref = SpatialRef::from_proj4(
            "+proj=geos +lon_0=42 +h=35785831 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs",
        )
        .unwrap();

        let central_meridian = spatial_ref
            .get_proj_param("central_meridian")
            .unwrap()
            .unwrap();
        assert_almost_eq(central_meridian, 42.0);

        let satellite_height = spatial_ref
            .get_proj_param("satellite_height")
            .unwrap()
            .unwrap();
        assert_almost_eq(satellite_height, 35_785_831.0);
    }

    #[test]
    fn setting_proj_param() {
        let mut spatial_ref = SpatialRef::from_proj4(
            "+proj=geos +lon_0=42 +h=35785831 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs",
        )
        .unwrap();

        spatial_ref
            .set_proj_param("central_meridian", -15.0)
            .unwrap();

        let central_meridian = spatial_ref
            .get_proj_param("central_meridian")
            .unwrap()
            .unwrap();

        assert_almost_eq(central_meridian, -15.0);
    }

    #[test]
    fn non_existing_proj_param() {
        let spatial_ref = SpatialRef::from_epsg(4326).unwrap();

        let spam = spatial_ref.get_proj_param("spam").unwrap();

        assert_eq!(spam, None);
    }

    #[test]
    fn attr_values() {
        let mut spatial_ref = SpatialRef::from_epsg(4326).unwrap();
        assert_eq!(
            spatial_ref.get_attr_value("GEOGCS", 0).unwrap().unwrap(),
            "WGS 84"
        );
        assert_eq!(
            spatial_ref
                .get_attr_value("GEOGCS|UNIT", 0)
                .unwrap()
                .unwrap(),
            "degree"
        );
        spatial_ref
            .set_attr_value("GEOGCS|UNIT", Some("meter"))
            .unwrap();
        assert_eq!(
            spatial_ref
                .get_attr_value("GEOGCS|UNIT", 0)
                .unwrap()
                .unwrap(),
            "meter"
        );
    }

    #[test]
    fn geog_cs() {
        let spatial_ref = SpatialRef::from_proj4(
            "+proj=geos +lon_0=42 +h=35785831 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs",
        )
        .unwrap();
        let expected_geog_cs = SpatialRef::from_wkt(
            r#"
                GEOGCS["unknown",
                    DATUM["WGS_1984",
                        SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],
                        AUTHORITY["EPSG","6326"]],
                    PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],
                    UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],
                    AXIS["Longitude",EAST],
                    AXIS["Latitude",NORTH]
                ]
            "#,
        )
        .unwrap();

        let geog_cs = spatial_ref.geog_cs().unwrap();

        assert_eq!(
            geog_cs,
            expected_geog_cs,
            "GEOGCS of geos spatial reference: \"{:?}\"\n does not equal to expected one: {:?}",
            geog_cs.to_wkt(),
            expected_geog_cs.to_wkt()
        );
    }
}
