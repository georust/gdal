use std::ffi::CString;

use gdal_sys::{self, CPLErr};

use crate::errors;
use crate::errors::*;
#[allow(unused)] // Referenced in doc comments.
use crate::spatial_ref::transform::CoordTransform;
use crate::utils::{_last_cpl_err, _last_null_pointer_err};

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
    pub fn new() -> errors::Result<CoordTransformOptions> {
        let c_obj = unsafe { gdal_sys::OCTNewCoordinateTransformationOptions() };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err(
                "OCTNewCoordinateTransformationOptions",
            ));
        }
        Ok(CoordTransformOptions { inner: c_obj })
    }

    /// Returns a C pointer to the allocated [`gdal_sys::OGRCoordinateTransformationOptionsH`] memory.
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub(crate) unsafe fn c_options(&self) -> gdal_sys::OGRCoordinateTransformationOptionsH {
        self.inner
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_ref::SpatialRef;

    #[test]
    #[cfg(any(major_ge_4, all(major_ge_3, minor_ge_3)))]
    fn invalid_transformation() {
        // This transformation can be constructed only if we allow ballpark transformations (enabled by
        // default).
        let ma = SpatialRef::from_epsg(6491).unwrap(); // Massachusetts
        let nl = SpatialRef::from_epsg(28992).unwrap(); // Netherlands
        let trafo = CoordTransform::new(&ma, &nl);
        assert!(trafo.is_ok());

        let mut options = CoordTransformOptions::new().unwrap();
        options.set_ballpark_allowed(false).unwrap();
        let trafo = CoordTransform::new_with_options(&ma, &nl, &options);
        let err = trafo.unwrap_err();
        assert!(matches!(err, GdalError::NullPointer { .. }), "{:?}", err);
    }

    #[test]
    fn set_coordinate_operation() {
        // Test case taken from:
        // https://gdal.org/tutorials/osr_api_tut.html#advanced-coordinate-transformation
        let mut options = CoordTransformOptions::new().unwrap();
        options
            .set_coordinate_operation("urn:ogc:def:coordinateOperation:EPSG::8599", false)
            .unwrap();
        let nad27 = SpatialRef::from_epsg(4267).unwrap();
        let wgs84 = SpatialRef::from_epsg(4326).unwrap();
        let trafo = CoordTransform::new_with_options(&nad27, &wgs84, &options);
        assert!(trafo.is_ok());
    }
}