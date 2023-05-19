//! Raster ground control point support

use crate::spatial_ref::SpatialRef;
use crate::utils::_string;
use crate::Dataset;

impl Dataset {
    /// Get output spatial reference system for GCPs.
    ///
    /// # Notes
    /// * This is separate and distinct from [`Dataset::spatial_ref`], and only applies to
    /// embedded GCPs.
    ///
    /// See: [`GDALGetGCPSpatialRef`](https://gdal.org/api/raster_c_api.html#_CPPv420GDALGetGCPSpatialRef12GDALDatasetH)
    pub fn gcp_spatial_ref(&self) -> Option<SpatialRef> {
        let c_ptr = unsafe { gdal_sys::GDALGetGCPSpatialRef(self.c_dataset()) };

        if c_ptr.is_null() {
            return None;
        }

        unsafe { SpatialRef::from_c_obj(c_ptr) }.ok()
    }

    /// Get the projection definition string for the GCPs in this dataset.
    ///
    /// # Notes
    /// * This is separate and distinct from [`Dataset::projection`], and only applies to
    /// embedded GCPs.
    ///
    ///  See: [`GDALGetGCPProjection`](https://gdal.org/api/raster_c_api.html#gdal_8h_1a85ffa184d3ecb7c0a59a66096b22b2ec)
    pub fn gcp_projection(&self) -> Option<String> {
        let cc_ptr = unsafe { gdal_sys::GDALGetGCPProjection(self.c_dataset()) };
        if cc_ptr.is_null() {
            return None;
        }
        Some(_string(cc_ptr))
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

    #[test]
    fn test_gcp_projection() {
        let dataset = Dataset::open(fixture("gcp.tif")).unwrap();
        let gcp_proj = dataset.gcp_projection();
        assert!(gcp_proj.is_some());
        assert!(gcp_proj.unwrap().contains("WGS 84"));
    }
}
