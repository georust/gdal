//! Raster ground control point support

use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use gdal_sys::CPLErr;

use crate::errors::Result;
use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_cpl_err, _string};
use crate::Dataset;

/// An owned Ground Control Point.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Gcp {
    /// Unique identifier, often numeric.
    pub id: String,
    /// Informational message or empty string.
    pub info: String,
    /// Pixel (x) location of GCP on raster.
    pub pixel: f64,
    /// Line (y) location of GCP on raster.
    pub line: f64,
    /// X position of GCP in georeferenced space.
    pub x: f64,
    /// Y position of GCP in georeferenced space.
    pub y: f64,
    /// Elevation of GCP, or zero if not known.
    pub z: f64,
}

/// A wrapper over a Ground Control Point, borrowed from an existing dataset.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct GcpRef<'a> {
    inner: gdal_sys::GDAL_GCP,
    _phantom: PhantomData<&'a gdal_sys::GDAL_GCP>,
}

impl GcpRef<'_> {
    /// Returns an unique identifier of the GCP, often numeric.
    pub fn id(&self) -> String {
        unsafe { CStr::from_ptr(self.inner.pszId) }
            .to_string_lossy()
            .into_owned()
    }

    /// Returns an informational message about the GCP, or an empty string.
    pub fn info(&self) -> String {
        unsafe { CStr::from_ptr(self.inner.pszInfo) }
            .to_string_lossy()
            .into_owned()
    }

    /// Returns the pixel (x) location of the GCP on the raster.
    pub fn pixel(&self) -> f64 {
        self.inner.dfGCPPixel
    }

    /// Returns the line (y) location of the GCP on the raster.
    pub fn line(&self) -> f64 {
        self.inner.dfGCPLine
    }

    /// Returns the X position of the GCP in georeferenced space.
    pub fn x(&self) -> f64 {
        self.inner.dfGCPX
    }

    /// Returns the Y position of the GCP in georeferenced space.
    pub fn y(&self) -> f64 {
        self.inner.dfGCPY
    }

    /// Returns the elevation of the GCP, or zero if not known.
    pub fn z(&self) -> f64 {
        self.inner.dfGCPZ
    }
}

impl From<&gdal_sys::GDAL_GCP> for Gcp {
    fn from(gcp: &gdal_sys::GDAL_GCP) -> Self {
        Gcp {
            id: _string(gcp.pszId),
            info: _string(gcp.pszId),
            pixel: gcp.dfGCPPixel,
            line: gcp.dfGCPLine,
            x: gcp.dfGCPX,
            y: gcp.dfGCPY,
            z: gcp.dfGCPZ,
        }
    }
}

impl From<&GcpRef<'_>> for Gcp {
    fn from(gcp: &GcpRef<'_>) -> Self {
        Gcp {
            id: gcp.id(),
            info: gcp.info(),
            pixel: gcp.pixel(),
            line: gcp.line(),
            x: gcp.x(),
            y: gcp.y(),
            z: gcp.z(),
        }
    }
}

impl Dataset {
    /// Get output spatial reference system for GCPs.
    ///
    /// # Notes
    /// * This is separate and distinct from [`Dataset::spatial_ref`], and only applies to
    ///   the representation of ground control points, when embedded.
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
    ///   embedded GCPs.
    ///
    ///  See: [`GDALGetGCPProjection`](https://gdal.org/api/raster_c_api.html#gdal_8h_1a85ffa184d3ecb7c0a59a66096b22b2ec)
    pub fn gcp_projection(&self) -> Option<String> {
        let cc_ptr = unsafe { gdal_sys::GDALGetGCPProjection(self.c_dataset()) };
        if cc_ptr.is_null() {
            return None;
        }
        Some(_string(cc_ptr))
    }

    /// Fetch GCPs.
    ///
    /// See: [`GDALDataset::GetGCPs`](https://gdal.org/api/gdaldataset_cpp.html#_CPPv4N11GDALDataset7GetGCPsEv)
    pub fn gcps(&self) -> &[GcpRef] {
        let len = unsafe { gdal_sys::GDALGetGCPCount(self.c_dataset()) };
        if len == 0 {
            return &[];
        }

        let data = unsafe { gdal_sys::GDALGetGCPs(self.c_dataset()) };
        unsafe { std::slice::from_raw_parts(data as *const GcpRef, len as usize) }
    }

    /// Assign GCPs.
    ///
    /// This method assigns the passed set of GCPs to this dataset, as well as
    /// setting their coordinate system.
    ///
    /// See: [`GDALDataset::SetGCPs(int, const GDAL_GCP *, const OGRSpatialReference *)`](https://gdal.org/api/gdaldataset_cpp.html#_CPPv4N11GDALDataset7SetGCPsEiPK8GDAL_GCPPK19OGRSpatialReference)
    ///
    /// # Panics
    ///
    /// Panics if `gcps` has more than [`std::ffi::c_int::MAX`] elements.
    pub fn set_gcps(&self, gcps: Vec<Gcp>, spatial_ref: &SpatialRef) -> Result<()> {
        let len = gcps
            .len()
            .try_into()
            .expect("only up to `INT_MAX` GCPs are supported");

        struct CGcp {
            id: CString,
            info: CString,
            pixel: f64,
            line: f64,
            x: f64,
            y: f64,
            z: f64,
        }

        let c_gcps = gcps
            .into_iter()
            .map(|gcp| {
                Ok(CGcp {
                    id: CString::new(gcp.id)?,
                    info: CString::new(gcp.info)?,
                    pixel: gcp.pixel,
                    line: gcp.line,
                    x: gcp.x,
                    y: gcp.y,
                    z: gcp.z,
                })
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();
        let gdal_gcps = c_gcps
            .iter()
            .map(|gcp| gdal_sys::GDAL_GCP {
                pszId: gcp.id.as_ptr() as *mut _,
                pszInfo: gcp.info.as_ptr() as *mut _,
                dfGCPPixel: gcp.pixel,
                dfGCPLine: gcp.line,
                dfGCPX: gcp.x,
                dfGCPY: gcp.y,
                dfGCPZ: gcp.z,
            })
            .collect::<Vec<_>>();

        let rv = unsafe {
            gdal_sys::GDALSetGCPs2(
                self.c_dataset(),
                len,
                gdal_gcps.as_ptr(),
                spatial_ref.to_c_hsrs() as *mut _,
            )
        };

        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Gcp;
    use crate::spatial_ref::SpatialRef;
    use crate::test_utils::{fixture, TempFixture};
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

    #[test]
    fn test_gcps() {
        let dataset = Dataset::open(fixture("gcp.tif")).unwrap();
        let gcps = dataset.gcps();
        assert_eq!(gcps.len(), 210);
        assert_eq!(gcps[0].id(), "1");
        assert_eq!(gcps[0].info(), "");
        assert_eq!(gcps[0].pixel(), 0.0);
        assert_eq!(gcps[0].line(), 0.0);
        assert_eq!(gcps[0].x(), -107.41653919575356);
        assert_eq!(gcps[0].y(), 45.02010727502759);
        assert_eq!(gcps[0].z(), 1260.933765466325);
        assert_eq!(gcps[209].id(), "210");
        assert_eq!(gcps[209].info(), "");
        assert_eq!(gcps[209].pixel(), 9.999614539567514);
        assert_eq!(gcps[209].line(), 5.999641105395382);
        assert_eq!(gcps[209].x(), -111.01882551533174);
        assert_eq!(gcps[209].y(), 43.92361434285831);
        assert_eq!(gcps[209].z(), 2265.000100511126);
    }

    #[test]
    #[cfg_attr(not(all(major_ge_3, minor_ge_5)), ignore)]
    fn test_set_gcps() {
        let fixture = TempFixture::fixture("gcp.tif");
        let dataset = Dataset::open(fixture).unwrap();
        let gcps = vec![
            Gcp {
                id: "1".to_owned(),
                info: "info 1".to_owned(),
                pixel: 0.0,
                line: 1.0,
                x: 100.0,
                y: 101.0,
                z: 50.0,
            },
            Gcp {
                id: "1".to_owned(),
                info: "info 2".to_owned(),
                pixel: 10.0,
                line: 11.0,
                x: 110.0,
                y: 111.0,
                z: 150.0,
            },
        ];
        let spatial_ref = SpatialRef::from_epsg(3857).unwrap();

        // The dataset is read-only, but we can still read them back from the PAM dataset.
        dataset.set_gcps(gcps.clone(), &spatial_ref).unwrap();

        let new_gcps = dataset.gcps().iter().map(Gcp::from).collect::<Vec<_>>();
        assert_eq!(new_gcps, gcps);
        let spatial_ref = dataset.gcp_spatial_ref().unwrap();
        assert_eq!(spatial_ref.auth_name().unwrap(), "EPSG");
        assert_eq!(spatial_ref.auth_code().unwrap(), 3857);
    }
}
