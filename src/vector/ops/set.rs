use crate::vector::Geometry;

/// # Set Operations
///
/// These methods provide set operations over two geometries, producing a new geometry.
impl Geometry {
    /// Computes the [geometric intersection][intersection] of `self` and `other`.
    ///
    /// Generates a new geometry which is the region of intersection of the two geometries operated on.
    ///
    /// # Notes
    /// * If you only need to determine if two geometries intersect and don't require
    /// the resultant region, use [`Geometry::intersects`].
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `None`.
    /// You  may check for GEOS support with [`VersionInfo::has_geos`][has_geos].
    ///
    /// # Returns
    /// * `Some(geometry)`: a new `Geometry` representing the computed intersection
    /// * `None`: when the geometries do not intersect or result could not be computed
    ///
    /// See: [`OGR_G_Intersection`][OGR_G_Intersection]
    ///
    /// [OGR_G_Intersection]: https://gdal.org/api/vector_c_api.html#_CPPv418OGR_G_Intersection12OGRGeometryH12OGRGeometryH
    /// [intersection]: https://en.wikipedia.org/wiki/Intersection_(geometry)
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.has_gdal_ptr() {
            return None;
        }
        if !other.has_gdal_ptr() {
            return None;
        }
        let ogr_geom =
            unsafe { gdal_sys::OGR_G_Intersection(self.c_geometry(), other.c_geometry()) };
        if ogr_geom.is_null() {
            return None;
        }
        Some(unsafe { Geometry::with_c_geometry(ogr_geom, true) })
    }

    /// Computes the [geometric union][union] of `self` and `other`.
    ///
    /// Generates a new geometry which is the union of the two geometries operated on.
    ///
    /// # Notes
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `None`.
    /// You  may check for GEOS support with [`VersionInfo::has_geos`][has_geos].
    ///
    /// # Returns
    /// * `Some(geometry)`: a new `Geometry` representing the computed union
    /// * `None`: when the union was not able to be computed
    ///
    /// See: [`OGR_G_Union`][OGR_G_Union]
    ///
    /// [OGR_G_Union]: https://gdal.org/api/vector_c_api.html#_CPPv411OGR_G_Union12OGRGeometryH12OGRGeometryH
    /// [union]: https://en.wikipedia.org/wiki/Constructive_solid_geometry#Workings
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn union(&self, other: &Self) -> Option<Self> {
        if !self.has_gdal_ptr() {
            return None;
        }
        if !other.has_gdal_ptr() {
            return None;
        }
        unsafe {
            let ogr_geom = gdal_sys::OGR_G_Union(self.c_geometry(), other.c_geometry());
            if ogr_geom.is_null() {
                return None;
            }
            Some(Geometry::with_c_geometry(ogr_geom, true))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_intersection_success() {
        let geom =
            Geometry::from_wkt("POLYGON ((0.0 10.0, 0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0))")
                .unwrap();
        let other =
            Geometry::from_wkt("POLYGON ((0.0 5.0, 0.0 0.0, 5.0 0.0, 5.0 5.0, 0.0 5.0))").unwrap();

        let inter = geom.intersection(&other);

        assert!(inter.is_some());

        let inter = inter.unwrap();

        assert_eq!(inter.area(), 25.0);
    }

    #[test]
    fn test_intersection_no_gdal_ptr() {
        let geom =
            Geometry::from_wkt("POLYGON ((0.0 10.0, 0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0))")
                .unwrap();
        let other = unsafe { Geometry::lazy_feature_geometry() };

        let inter = geom.intersection(&other);

        assert!(inter.is_none());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_intersection_no_intersects() {
        let geom =
            Geometry::from_wkt("POLYGON ((0.0 5.0, 0.0 0.0, 5.0 0.0, 5.0 5.0, 0.0 5.0))").unwrap();

        let other =
            Geometry::from_wkt("POLYGON ((15.0 15.0, 15.0 20.0, 20.0 20.0, 20.0 15.0, 15.0 15.0))")
                .unwrap();

        let inter = geom.intersection(&other);

        assert!(inter.is_some());

        assert_eq!(inter.unwrap().area(), 0.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_union_success() {
        let geom =
            Geometry::from_wkt("POLYGON ((0.0 10.0, 0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0))")
                .unwrap();
        let other = Geometry::from_wkt("POLYGON ((1 -5, 1 1, -5 1, -5 -5, 1 -5))").unwrap();

        let res = geom.union(&other);

        assert!(res.is_some());

        let res = res.unwrap();

        assert_eq!(res.area(), 135.0);
    }

    #[test]
    fn test_union_no_gdal_ptr() {
        let geom =
            Geometry::from_wkt("POLYGON ((0.0 10.0, 0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0))")
                .unwrap();
        let other = unsafe { Geometry::lazy_feature_geometry() };

        let res = geom.union(&other);

        assert!(res.is_none());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_union_no_intersects() {
        let geom =
            Geometry::from_wkt("POLYGON ((0.0 5.0, 0.0 0.0, 5.0 0.0, 5.0 5.0, 0.0 5.0))").unwrap();

        let other =
            Geometry::from_wkt("POLYGON ((15.0 15.0, 15.0 20.0, 20.0 20.0, 20.0 15.0, 15.0 15.0))")
                .unwrap();

        let res = geom.union(&other);

        assert!(res.is_some());

        assert_eq!(res.unwrap().area(), 50.0);
    }
}
