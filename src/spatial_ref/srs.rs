use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string};
use gdal_sys::{self, CPLErr, OGRCoordinateTransformationH, OGRErr, OGRSpatialReferenceH};
use libc::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::ptr::{self, null_mut};
use std::str::FromStr;

use crate::errors::*;

/// Options for [`CoordTransform::new_with_options`].
#[derive(Debug)]
pub struct CoordTransformOptions {
    inner: gdal_sys::OGRCoordinateTransformationOptionsH,
}

impl Drop for CoordTransformOptions {
    fn drop(&mut self) {
        unsafe { gdal_sys::OCTDestroyCoordinateTransformationOptions(self.inner) };
    }
}

impl CoordTransformOptions {
    /// Creation options for [`CoordTransform`].
    pub fn new() -> Result<CoordTransformOptions> {
        let c_obj = unsafe { gdal_sys::OCTNewCoordinateTransformationOptions() };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err(
                "OCTNewCoordinateTransformationOptions",
            ));
        }
        Ok(CoordTransformOptions { inner: c_obj })
    }

    /// Sets an area of interest.
    ///
    /// The west longitude is generally lower than the east longitude, except for areas of interest
    /// that go across the anti-meridian.
    ///
    /// For more information, see:
    /// <https://gdal.org/tutorials/osr_api_tut.html#advanced-coordinate-transformation>
    ///
    /// # Arguments
    ///
    /// - `west_longitude_deg` – West longitude (in degree). Must be in [-180,180]
    /// - `south_latitude_deg` – South latitude (in degree). Must be in [-90,90]
    /// - `east_longitude_deg` – East longitude (in degree). Must be in [-180,180]
    /// - `north_latitude_deg` – North latitude (in degree). Must be in [-90,90]
    pub fn set_area_of_interest(
        &mut self,
        west_longitude_deg: f64,
        south_latitude_deg: f64,
        east_longitude_deg: f64,
        north_latitude_deg: f64,
    ) -> Result<()> {
        let ret_val = unsafe {
            gdal_sys::OCTCoordinateTransformationOptionsSetAreaOfInterest(
                self.inner,
                west_longitude_deg,
                south_latitude_deg,
                east_longitude_deg,
                north_latitude_deg,
            )
        };
        if ret_val == 0 {
            return Err(_last_cpl_err(CPLErr::CE_Failure));
        }
        Ok(())
    }

    /// Sets the desired accuracy for coordinate operations.
    ///
    /// Only coordinate operations that offer an accuracy of at least the one specified will be
    /// considered.
    ///
    /// An accuracy of 0 is valid and means a coordinate operation made only of one or several
    /// conversions (map projections, unit conversion, etc.) Operations involving ballpark
    /// transformations have a unknown accuracy, and will be filtered out by any dfAccuracy >= 0
    /// value.
    ///
    /// If this option is specified with PROJ < 8, the `OGR_CT_OP_SELECTION` configuration option
    /// will default to `BEST_ACCURACY`.
    #[cfg(any(major_ge_4, all(major_ge_3, minor_ge_3)))]
    pub fn desired_accuracy(&mut self, accuracy: f64) -> Result<()> {
        let ret_val = unsafe {
            gdal_sys::OCTCoordinateTransformationOptionsSetDesiredAccuracy(self.inner, accuracy)
        };
        if ret_val == 0 {
            return Err(_last_cpl_err(CPLErr::CE_Failure));
        }
        Ok(())
    }

    /// Sets whether ballpark transformations are allowed.
    ///
    /// By default, PROJ may generate "ballpark transformations" (see
    /// <https://proj.org/glossary.html>) when precise datum transformations are missing. For high
    /// accuracy use cases, such transformations might not be allowed.
    ///
    /// If this option is specified with PROJ < 8, the `OGR_CT_OP_SELECTION` configuration option
    /// will default to `BEST_ACCURACY`.
    #[cfg(any(major_ge_4, all(major_ge_3, minor_ge_3)))]
    pub fn set_ballpark_allowed(&mut self, ballpark_allowed: bool) -> Result<()> {
        let ret_val = unsafe {
            gdal_sys::OCTCoordinateTransformationOptionsSetBallparkAllowed(
                self.inner,
                ballpark_allowed as libc::c_int,
            )
        };
        if ret_val == 0 {
            return Err(_last_cpl_err(CPLErr::CE_Failure));
        }
        Ok(())
    }

    /// Sets a coordinate operation.
    ///
    /// This is a user override to be used instead of the normally computed pipeline.
    ///
    /// The pipeline must take into account the axis order of the source and target SRS.
    ///
    /// The pipeline may be provided as a PROJ string (single step operation or multiple step
    /// string starting with `+proj=pipeline`), a WKT2 string describing a `CoordinateOperation`,
    /// or a `"urn:ogc:def:coordinateOperation:EPSG::XXXX"` URN.
    ///
    /// For more information, see:
    /// <https://gdal.org/tutorials/osr_api_tut.html#advanced-coordinate-transformation>
    ///
    /// # Arguments
    ///
    /// - `co`: PROJ or WKT string describing a coordinate operation
    /// - `reverse`: Whether the PROJ or WKT string should be evaluated in the reverse path
    pub fn set_coordinate_operation(&mut self, co: &str, reverse: bool) -> Result<()> {
        let c_co = CString::new(co)?;
        let ret_val = unsafe {
            gdal_sys::OCTCoordinateTransformationOptionsSetOperation(
                self.inner,
                c_co.as_ptr(),
                reverse as libc::c_int,
            )
        };
        if ret_val == 0 {
            return Err(_last_cpl_err(CPLErr::CE_Failure));
        }
        Ok(())
    }
}

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
        Ok(CoordTransform {
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
            gdal_sys::OCTNewCoordinateTransformationEx(source.0, target.0, options.inner)
        };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OCTNewCoordinateTransformation"));
        }
        Ok(CoordTransform {
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
