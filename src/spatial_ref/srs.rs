use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string};
use gdal_sys::{self, CPLErr, OGRCoordinateTransformationH, OGRErr, OGRSpatialReferenceH};
use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::ptr::{self, null_mut};
use std::str::FromStr;

use crate::errors::*;
use crate::spatial_ref::transform_opts::CoordTransformOptions;

#[derive(Debug)]
pub struct CoordTransform {
    inner: OGRCoordinateTransformationH,
    from: String,
    to: String,
}

impl Drop for CoordTransform {
    fn drop(&mut self) {
        unsafe { gdal_sys::OCTDestroyCoordinateTransformation(self.inner) };
    }
}

impl CoordTransform {
    pub fn new(source: &SpatialRef, target: &SpatialRef) -> Result<CoordTransform> {
        let c_obj = unsafe { gdal_sys::OCTNewCoordinateTransformation(source.0, target.0) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OCTNewCoordinateTransformation"));
        }
        Ok(Self {
            inner: c_obj,
            from: source.authority().or_else(|_| source.to_proj4())?,
            to: target.authority().or_else(|_| target.to_proj4())?,
        })
    }

    pub fn new_with_options(
        source: &SpatialRef,
        target: &SpatialRef,
        options: &CoordTransformOptions,
    ) -> Result<CoordTransform> {
        let c_obj = unsafe {
            gdal_sys::OCTNewCoordinateTransformationEx(source.0, target.0, options.c_options())
        };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OCTNewCoordinateTransformation"));
        }
        Ok(Self {
            inner: c_obj,
            from: source.authority().or_else(|_| source.to_proj4())?,
            to: target.authority().or_else(|_| target.to_proj4())?,
        })
    }

    /// Transform bounding box, densifying the edges to account for nonlinear
    /// transformations.
    ///
    /// # Arguments
    /// * `bounds` - array of [axis0_min, axis1_min, axis0_max, axis1_max],
    ///            interpreted in the axis order of the source SpatialRef,
    ///            typically [xmin, ymin, xmax, ymax]
    /// * `densify_pts` - number of points per edge (recommended: 21)
    ///
    /// # Returns
    /// `Ok([f64; 4])` with bounds in axis order of target SpatialRef
    /// `Err` if there is an error.
    #[cfg(all(major_ge_3, minor_ge_4))]
    pub fn transform_bounds(&self, bounds: &[f64; 4], densify_pts: i32) -> Result<[f64; 4]> {
        let mut out_xmin: f64 = 0.;
        let mut out_ymin: f64 = 0.;
        let mut out_xmax: f64 = 0.;
        let mut out_ymax: f64 = 0.;

        let ret_val = unsafe {
            gdal_sys::OCTTransformBounds(
                self.inner,
                bounds[0],
                bounds[1],
                bounds[2],
                bounds[3],
                &mut out_xmin,
                &mut out_ymin,
                &mut out_xmax,
                &mut out_ymax,
                densify_pts as c_int,
            ) == 1
        };

        if !ret_val {
            let msg = match _last_cpl_err(CPLErr::CE_Failure) {
                GdalError::CplError { msg, .. } => match msg.is_empty() {
                    false => Some(msg),
                    _ => None,
                },
                err => return Err(err),
            };
            return Err(GdalError::InvalidCoordinateRange {
                from: self.from.clone(),
                to: self.to.clone(),
                msg,
            });
        }

        Ok([out_xmin, out_ymin, out_xmax, out_ymax])
    }

    /// Transform coordinates in place.
    ///
    /// # Arguments
    /// * `x` - slice of x coordinates
    /// * `y` - slice of y coordinates (must match x in length)
    /// * `z` - slice of z coordinates, or an empty slice to ignore
    pub fn transform_coords(&self, x: &mut [f64], y: &mut [f64], z: &mut [f64]) -> Result<()> {
        let nb_coords = x.len();
        assert_eq!(
            nb_coords,
            y.len(),
            "transform coordinate slices have different lengths: {} != {}",
            nb_coords,
            y.len()
        );
        let ret_val = unsafe {
            gdal_sys::OCTTransform(
                self.inner,
                nb_coords as c_int,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                if z.is_empty() {
                    null_mut()
                } else {
                    assert_eq!(
                        nb_coords,
                        z.len(),
                        "transform coordinate slices have different lengths: {} != {}",
                        nb_coords,
                        z.len()
                    );
                    z.as_mut_ptr()
                },
            ) == 1
        };

        if ret_val {
            Ok(())
        } else {
            let err = _last_cpl_err(CPLErr::CE_Failure);
            let msg = if let GdalError::CplError { msg, .. } = err {
                if msg.trim().is_empty() {
                    None
                } else {
                    Some(msg)
                }
            } else {
                return Err(err);
            };
            Err(GdalError::InvalidCoordinateRange {
                from: self.from.clone(),
                to: self.to.clone(),
                msg,
            })
        }
    }

    #[deprecated(since = "0.3.1", note = "use `transform_coords` instead")]
    pub fn transform_coord(&self, x: &mut [f64], y: &mut [f64], z: &mut [f64]) {
        self.transform_coords(x, y, z)
            .expect("Coordinate transform failed")
    }

    pub fn to_c_hct(&self) -> OGRCoordinateTransformationH {
        self.inner
    }
}

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
pub struct SpatialRef(OGRSpatialReferenceH);

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

    /// Returns a wrapped `SpatialRef` from a raw C API handle.
    ///
    /// # Safety
    /// The handle passed to this function must be valid.
    pub unsafe fn from_c_obj(c_obj: OGRSpatialReferenceH) -> Result<SpatialRef> {
        let mut_c_obj = gdal_sys::OSRClone(c_obj);
        if mut_c_obj.is_null() {
            Err(_last_null_pointer_err("OSRClone"))
        } else {
            Ok(SpatialRef(mut_c_obj))
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

    // TODO: should this take self instead of &self?
    pub fn to_c_hsrs(&self) -> OGRSpatialReferenceH {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_almost_eq;
    use crate::errors::GdalError;
    use crate::vector::Geometry;

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
            "{:?} does not contain expected CRS name",
            projjson
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

        assert!(spatial_ref1 == spatial_ref2);
        assert!(spatial_ref2 != spatial_ref3);
        assert!(spatial_ref4 == spatial_ref2);
        assert!(spatial_ref5 == spatial_ref4);
    }

    #[cfg(all(major_ge_3, minor_ge_4))]
    #[test]
    fn transform_bounds() {
        let bounds: [f64; 4] = [-180., -80., 180., 80.];
        // bounds for y,x ordered SpatialRefs
        let yx_bounds: [f64; 4] = [-80.0, -180.0, 80.0, 180.];

        let spatial_ref1 = SpatialRef::from_definition("OGC:CRS84").unwrap();

        // transforming between the same SpatialRef should return existing bounds
        let mut transform = CoordTransform::new(&spatial_ref1, &spatial_ref1).unwrap();
        let mut out_bounds = transform.transform_bounds(&bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], bounds[0]);
        assert_almost_eq(out_bounds[1], bounds[1]);
        assert_almost_eq(out_bounds[2], bounds[2]);
        assert_almost_eq(out_bounds[3], bounds[3]);

        // EPSG:4326 is in y,x order by default; returned bounds are [ymin, xmin, ymax, xmax]
        let mut spatial_ref2 = SpatialRef::from_epsg(4326).unwrap();
        transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        out_bounds = transform.transform_bounds(&bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], yx_bounds[0]);
        assert_almost_eq(out_bounds[1], yx_bounds[1]);
        assert_almost_eq(out_bounds[2], yx_bounds[2]);
        assert_almost_eq(out_bounds[3], yx_bounds[3]);

        // if source SpatialRef is in y,x order and and target SpatialRef is in x,y order
        // input bounds are interpreted as [ymin, xmin, ymax, xmax] and returns
        // [xmin, ymin, xmax, ymax]
        transform = CoordTransform::new(&spatial_ref2, &spatial_ref1).unwrap();
        out_bounds = transform.transform_bounds(&yx_bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], bounds[0]);
        assert_almost_eq(out_bounds[1], bounds[1]);
        assert_almost_eq(out_bounds[2], bounds[2]);
        assert_almost_eq(out_bounds[3], bounds[3]);

        // force EPSG:4326 into x,y order to match source SpatialRef
        spatial_ref2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        out_bounds = transform.transform_bounds(&bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], bounds[0]);
        assert_almost_eq(out_bounds[1], bounds[1]);
        assert_almost_eq(out_bounds[2], bounds[2]);
        assert_almost_eq(out_bounds[3], bounds[3]);

        spatial_ref2 = SpatialRef::from_epsg(3857).unwrap();
        transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        out_bounds = transform.transform_bounds(&bounds, 21).unwrap();

        let expected_bounds: [f64; 4] = [
            -20037508.342789244,
            -15538711.096309224,
            20037508.342789244,
            15538711.09630923,
        ];
        assert_almost_eq(out_bounds[0], expected_bounds[0]);
        assert_almost_eq(out_bounds[1], expected_bounds[1]);
        assert_almost_eq(out_bounds[2], expected_bounds[2]);
        assert_almost_eq(out_bounds[3], expected_bounds[3]);
    }

    #[test]
    fn transform_coordinates() {
        let spatial_ref1 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
        let spatial_ref2 = SpatialRef::from_epsg(3035).unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        spatial_ref1.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        spatial_ref2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        let mut xs = [23.43, 23.50];
        let mut ys = [37.58, 37.70];
        let mut zs = [32.0, 20.0];
        transform
            .transform_coords(&mut xs, &mut ys, &mut zs)
            .unwrap();
        assert_almost_eq(xs[0], 5509543.1508097);
        assert_almost_eq(ys[0], 1716062.1916192223);
        assert_almost_eq(zs[0], 32.0);
    }

    #[test]
    fn transform_ogr_geometry() {
        //let expected_value = "POLYGON ((5509543.150809700600803 1716062.191619219258428,5467122.000330002978444 1980151.204280239529908,5623571.028492723591626 2010213.310253676958382,5671834.921544363722205 1746968.078280254499987,5509543.150809700600803 1716062.191619219258428))";
        //let expected_value = "POLYGON ((5509543.15080969966948 1716062.191619222285226,5467122.000330002047122 1980151.204280242323875,5623571.028492721728981 2010213.31025367998518,5671834.921544362790883 1746968.078280256595463,5509543.15080969966948 1716062.191619222285226))";
        let expected_value = "POLYGON ((5509543.1508097 1716062.19161922,5467122.00033 1980151.20428024,5623571.02849272 2010213.31025368,5671834.92154436 1746968.07828026,5509543.1508097 1716062.19161922))";
        let mut geom = Geometry::from_wkt(
            "POLYGON((23.43 37.58, 23.43 40.0, 25.29 40.0, 25.29 37.58, 23.43 37.58))",
        )
        .unwrap();
        let spatial_ref1 = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
        let spatial_ref2 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        spatial_ref1.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        spatial_ref2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let htransform = CoordTransform::new(&spatial_ref2, &spatial_ref1).unwrap();
        geom.transform_inplace(&htransform).unwrap();
        assert_eq!(expected_value, geom.wkt().unwrap());
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
    fn failing_transformation() {
        let wgs84 = SpatialRef::from_epsg(4326).unwrap();
        let dhd_2 = SpatialRef::from_epsg(31462).unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        wgs84.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        dhd_2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let mut x = [1979105.06, 0.0];
        let mut y = [5694052.67, 0.0];
        let mut z = [0.0, 0.0];

        let trafo = CoordTransform::new(&wgs84, &dhd_2).unwrap();
        let r = trafo.transform_coords(&mut x, &mut y, &mut z);
        assert!(r.is_err());

        let wgs84 = SpatialRef::from_epsg(4326).unwrap();
        let webmercator = SpatialRef::from_epsg(3857).unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        wgs84.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        webmercator.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let mut x = [1000000.0];
        let mut y = [1000000.0];

        let trafo = CoordTransform::new(&wgs84, &webmercator).unwrap();
        let r = trafo.transform_coords(&mut x, &mut y, &mut []);

        assert!(r.is_err());
        if let GdalError::InvalidCoordinateRange { .. } = r.unwrap_err() {
            // assert_eq!(msg, &Some("latitude or longitude exceeded limits".into()));
        } else {
            panic!("Wrong error type");
        }
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
