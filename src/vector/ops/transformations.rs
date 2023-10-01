use gdal_sys::OGRErr;

use crate::cpl::CslStringList;
use crate::errors::{GdalError, Result};
use crate::spatial_ref::CoordTransform;
use crate::spatial_ref::SpatialRef;
use crate::utils::_last_null_pointer_err;
use crate::vector::Geometry;

/// # Geometry Transformations
///
/// These methods provide geometric transformations on a `Geometry`.
impl Geometry {
    /// Apply arbitrary coordinate transformation to geometry, mutating the [`Geometry`] in-place.
    ///
    /// See: [`OGR_G_Transform`](https://gdal.org/api/vector_c_api.html#_CPPv415OGR_G_Transform12OGRGeometryH28OGRCoordinateTransformationH)
    pub fn transform_inplace(&mut self, htransform: &CoordTransform) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_G_Transform(self.c_geometry(), htransform.to_c_hct()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_Transform",
            });
        }
        Ok(())
    }

    /// Apply arbitrary coordinate transformation to geometry on a clone of `Self`.
    ///
    /// See: [`OGR_G_Transform`](https://gdal.org/api/vector_c_api.html#_CPPv415OGR_G_Transform12OGRGeometryH28OGRCoordinateTransformationH)
    pub fn transform(&self, htransform: &CoordTransform) -> Result<Geometry> {
        let new_c_geom = unsafe { gdal_sys::OGR_G_Clone(self.c_geometry()) };
        let rv = unsafe { gdal_sys::OGR_G_Transform(new_c_geom, htransform.to_c_hct()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_Transform",
            });
        }
        Ok(unsafe { Geometry::with_c_geometry(new_c_geom, true) })
    }

    /// Transforms this geometry's coordinates into another [`SpatialRef`], mutating the [`Geometry`] in-place.
    ///
    /// See: [`OGR_G_TransformTo`](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_TransformTo12OGRGeometryH20OGRSpatialReferenceH)
    pub fn transform_to_inplace(&mut self, spatial_ref: &SpatialRef) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_G_TransformTo(self.c_geometry(), spatial_ref.to_c_hsrs()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_TransformTo",
            });
        }
        Ok(())
    }

    /// Transforms this geometry's coordinates into another [`SpatialRef`].
    ///
    /// See: [`OGR_G_TransformTo`](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_TransformTo12OGRGeometryH20OGRSpatialReferenceH)
    pub fn transform_to(&self, spatial_ref: &SpatialRef) -> Result<Geometry> {
        let new_c_geom = unsafe { gdal_sys::OGR_G_Clone(self.c_geometry()) };
        let rv = unsafe { gdal_sys::OGR_G_TransformTo(new_c_geom, spatial_ref.to_c_hsrs()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_TransformTo",
            });
        }
        Ok(unsafe { Geometry::with_c_geometry(new_c_geom, true) })
    }

    /// Compute the convex hull of this geometry.
    ///
    /// See: [`OGR_G_ConvexHull`](https://gdal.org/api/vector_c_api.html#_CPPv416OGR_G_ConvexHull12OGRGeometryH)
    pub fn convex_hull(&self) -> Result<Geometry> {
        let c_geom = unsafe { gdal_sys::OGR_G_ConvexHull(self.c_geometry()) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_ConvexHull"));
        };
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    #[cfg(any(all(major_is_2, minor_ge_1), major_ge_3))]
    /// Return a [Delaunay triangulation of][dt] the vertices of the geometry.
    ///
    /// # Arguments
    /// * `tolerance`: optional snapping tolerance to use for improved robustness
    ///
    /// # Notes
    /// This function requires GEOS library, v3.4 or above.
    /// If OGR is built without the GEOS library, this function will always fail.
    /// Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_DelaunayTriangulation`](https://gdal.org/api/vector_c_api.html#_CPPv427OGR_G_DelaunayTriangulation12OGRGeometryHdi)
    ///
    /// [dt]: https://en.wikipedia.org/wiki/Delaunay_triangulation
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn delaunay_triangulation(&self, tolerance: Option<f64>) -> Result<Self> {
        let c_geom = unsafe {
            gdal_sys::OGR_G_DelaunayTriangulation(self.c_geometry(), tolerance.unwrap_or(0.0), 0)
        };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_DelaunayTriangulation"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Compute a simplified geometry.
    ///
    /// # Arguments
    /// * `tolerance`: the distance tolerance for the simplification.
    ///
    /// See: [`OGR_G_Simplify`](https://gdal.org/api/vector_c_api.html#_CPPv414OGR_G_Simplify12OGRGeometryHd)
    pub fn simplify(&self, tolerance: f64) -> Result<Self> {
        let c_geom = unsafe { gdal_sys::OGR_G_Simplify(self.c_geometry(), tolerance) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_Simplify"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Simplify the geometry while preserving topology.
    ///
    /// # Arguments
    /// * `tolerance`: the distance tolerance for the simplification.
    ///
    /// See: [`OGR_G_SimplifyPreserveTopology`](https://gdal.org/api/vector_c_api.html#_CPPv430OGR_G_SimplifyPreserveTopology12OGRGeometryHd)
    pub fn simplify_preserve_topology(&self, tolerance: f64) -> Result<Self> {
        let c_geom =
            unsafe { gdal_sys::OGR_G_SimplifyPreserveTopology(self.c_geometry(), tolerance) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_SimplifyPreserveTopology"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Compute buffer of geometry
    ///
    /// # Arguments
    /// * `distance`: the buffer distance to be applied. Should be expressed in
    ///   the same unit as the coordinates of the geometry.
    /// * `n_quad_segs` specifies the number of segments used to approximate a
    ///   90 degree (quadrant) of curvature.
    ///
    /// See: [`OGR_G_Buffer`](https://gdal.org/api/vector_c_api.html#_CPPv412OGR_G_Buffer12OGRGeometryHdi)
    pub fn buffer(&self, distance: f64, n_quad_segs: u32) -> Result<Self> {
        let c_geom =
            unsafe { gdal_sys::OGR_G_Buffer(self.c_geometry(), distance, n_quad_segs as i32) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_Buffer"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Attempts to make an invalid geometry valid without losing vertices.
    ///
    /// Already-valid geometries are cloned without further intervention.
    ///
    /// Extended options are available via [`CslStringList`] if GDAL is built with GEOS >= 3.8.
    /// They are defined as follows:
    ///
    /// * `METHOD=LINEWORK`: Combines all rings into a set of node-ed lines and then extracts
    ///    valid polygons from that "linework".
    /// * `METHOD=STRUCTURE`: First makes all rings valid, then merges shells and subtracts holes
    ///    from shells to generate valid result. Assumes holes and shells are correctly categorized.
    /// * `KEEP_COLLAPSED=YES/NO`. Only for `METHOD=STRUCTURE`.
    ///   - `NO` (default):  Collapses are converted to empty geometries
    ///   - `YES`: collapses are converted to a valid geometry of lower dimension
    ///
    /// When GEOS < 3.8, this method will return `Ok(self.clone())` if it is valid, or `Err` if not.
    ///
    /// See: [OGR_G_MakeValidEx](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_MakeValidEx12OGRGeometryH12CSLConstList)
    ///
    /// # Example
    /// ```rust, no_run
    /// use gdal::cpl::CslStringList;
    /// use gdal::vector::Geometry;
    /// # fn main() -> gdal::errors::Result<()> {
    /// let src = Geometry::from_wkt("POLYGON ((0 0, 10 10, 0 10, 10 0, 0 0))")?;
    /// let dst = src.make_valid(&CslStringList::new())?;
    /// assert_eq!("MULTIPOLYGON (((10 0, 0 0, 5 5, 10 0)),((10 10, 5 5, 0 10, 10 10)))", dst.wkt()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_valid(&self, opts: &CslStringList) -> Result<Geometry> {
        #[cfg(all(major_ge_3, minor_ge_4))]
        let c_geom = unsafe { gdal_sys::OGR_G_MakeValidEx(self.c_geometry(), opts.as_ptr()) };

        #[cfg(not(all(major_ge_3, minor_ge_4)))]
        let c_geom = {
            if !opts.is_empty() {
                return Err(GdalError::BadArgument(
                    "Options to make_valid require GDAL >= 3.4".into(),
                ));
            }
            unsafe { gdal_sys::OGR_G_MakeValid(self.c_geometry()) }
        };

        if c_geom.is_null() {
            Err(_last_null_pointer_err("OGR_G_MakeValid"))
        } else {
            Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::SuppressGDALErrorLog;

    #[test]
    fn test_convex_hull() {
        let star = "POLYGON ((0 1,3 1,1 3,1.5 0.0,2 3,0 1))";
        let hull = "POLYGON ((1.5 0.0,0 1,1 3,2 3,3 1,1.5 0.0))";
        assert_eq!(
            Geometry::from_wkt(star)
                .unwrap()
                .convex_hull()
                .unwrap()
                .wkt()
                .unwrap(),
            hull
        );
    }

    #[test]
    #[cfg(any(all(major_is_2, minor_ge_1), major_ge_3))]
    fn test_delaunay_triangulation() -> Result<()> {
        let square = Geometry::from_wkt("POLYGON ((0 1,1 1,1 0,0 0,0 1))")?;
        let triangles = Geometry::from_wkt(
            "GEOMETRYCOLLECTION (POLYGON ((0 1,0 0,1 0,0 1)),POLYGON ((0 1,1 0,1 1,0 1)))",
        )?;
        assert_eq!(square.delaunay_triangulation(None)?, triangles);
        Ok(())
    }

    #[test]
    fn test_simplify() -> Result<()> {
        let line = Geometry::from_wkt("LINESTRING(1.2 0.19,1.63 0.58,1.98 0.65,2.17 0.89)")?;
        let triangles = Geometry::from_wkt("LINESTRING (1.2 0.19,2.17 0.89)")?;
        assert_eq!(line.simplify(0.5)?, triangles);
        Ok(())
    }

    #[test]
    fn test_simplify_preserve_topology() -> Result<()> {
        let test = Geometry::from_wkt("LINESTRING(0 0,1 0,10 0)")?;
        let expected = Geometry::from_wkt("LINESTRING (0 0,10 0)")?;
        assert_eq!(test.simplify_preserve_topology(5.0)?, expected);
        Ok(())
    }

    #[test]
    pub fn test_buffer() {
        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        let buffered = geom.buffer(10.0, 2).unwrap();
        assert_eq!(
            buffered.geometry_type(),
            ::gdal_sys::OGRwkbGeometryType::wkbPolygon
        );
        assert!(buffered.area() > 10.0);
    }

    #[test]
    /// Simple clone case.
    pub fn test_make_valid_clone() {
        let src = Geometry::from_wkt("POINT (0 0)").unwrap();
        let dst = src.make_valid(&CslStringList::new());
        assert!(dst.is_ok());
        assert!(dst.unwrap().is_valid());
    }

    #[test]
    /// Un-repairable geometry case
    pub fn test_make_valid_invalid() {
        let _nolog = SuppressGDALErrorLog::new();
        let src = Geometry::from_wkt("LINESTRING (0 0)").unwrap();
        assert!(!src.is_valid());
        let dst = src.make_valid(&CslStringList::new());
        assert!(dst.is_err());
    }

    #[test]
    /// Repairable case (self-intersecting)
    pub fn test_make_valid_repairable() {
        let src = Geometry::from_wkt("POLYGON ((0 0, 10 10, 0 10, 10 0, 0 0))").unwrap();
        assert!(!src.is_valid());
        let dst = src.make_valid(&CslStringList::new());
        assert!(dst.is_ok());
        assert!(dst.unwrap().is_valid());
    }

    #[cfg(all(major_ge_3, minor_ge_4))]
    #[test]
    /// Repairable case, but use extended options
    pub fn test_make_valid_ex() {
        let src =
            Geometry::from_wkt("POLYGON ((0 0, 0 10, 10 10, 10 0, 0 0),(5 5, 15 10, 15 0, 5 5))")
                .unwrap();
        let opts = CslStringList::try_from(&[("STRUCTURE", "LINEWORK")]).unwrap();
        let dst = src.make_valid(&opts);
        assert!(dst.is_ok(), "{dst:?}");
        assert!(dst.unwrap().is_valid());
    }
}
