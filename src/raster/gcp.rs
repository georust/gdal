//! Raster ground control point support

use crate::spatial_ref::SpatialRef;
use crate::Dataset;

impl Dataset {
    /// Get output spatial reference system for GCPs.
    ///
    /// # Notes
    /// * This is separate and distinct from [`Dataset::spatial_ref`], and only applies to
    /// the representation of ground control points, when embedded.
    ///
    /// See: [`GDALGetGCPSpatialRef`](https://gdal.org/api/raster_c_api.html#_CPPv420GDALGetGCPSpatialRef12GDALDatasetH)
    pub fn gcp_spatial_ref(&self) -> Option<SpatialRef> {
        let c_ptr = unsafe { gdal_sys::GDALGetGCPSpatialRef(self.c_dataset()) };

        if c_ptr.is_null() {
            return None;
        }

        unsafe { SpatialRef::from_c_obj(c_ptr) }.ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::fixture;
    use crate::Dataset;

    #[test]
    fn test_gcp_spatial_ref() {
        let dataset = Dataset::open(fixture("gcp.tif")).unwrap();
        let gcp_srs = dataset.gcp_spatial_ref();
        let auth = gcp_srs.and_then(|s| s.authority().ok());
        assert_eq!(auth.unwrap(), "EPSG:4326");
    }
}
