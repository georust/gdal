use crate::vector::Geometry;
use foreign_types::ForeignType;

/// # Geometric Predicates
///
/// These methods provide common [spatial relations](https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates) between
/// two geometries.
impl Geometry {
    /// Tests if two geometries [_intersect_][DE-9IM];
    /// `self` and `other` have at least one point in common.
    ///
    /// If GEOS is enabled, then this is done in rigorous fashion, otherwise `true` is returned
    /// if the envelopes (bounding boxes) of the two geometries overlap. Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_Intersects`][OGR_G_Intersects]
    ///
    /// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates
    /// [OGR_G_Intersects]: https://gdal.org/api/vector_c_api.html#ogr__api_8h_1acaed6926b75cd33a42b284c10def6e87
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn intersects(&self, other: &Self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_Intersects(self.as_ptr(), other.as_ptr()) };
        p != 0
    }

    /// Tests if this geometry [_contains_][DE-9IM] the other geometry;
    /// `other` lies in `self`, and the interiors intersect.
    ///
    /// # Notes
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `false`. Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_Contains`][OGR_G_Contains]
    ///
    /// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates
    /// [OGR_G_Contains]: https://gdal.org/api/vector_c_api.html#_CPPv414OGR_G_Contains12OGRGeometryH12OGRGeometryH
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn contains(&self, other: &Self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_Contains(self.as_ptr(), other.as_ptr()) };
        p != 0
    }

    /// Tests if this geometry and the other geometry are [_disjoint_][DE-9IM];
    /// `self` and `other` form a set of disconnected geometries.
    ///
    /// # Notes
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `false`. Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_Disjoint`][OGR_G_Disjoint]
    ///
    /// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates
    /// [OGR_G_Disjoint]: https://gdal.org/api/vector_c_api.html#_CPPv414OGR_G_Disjoint12OGRGeometryH12OGRGeometryH
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn disjoint(&self, other: &Self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_Disjoint(self.as_ptr(), other.as_ptr()) };
        p != 0
    }

    /// Tests if this geometry and the other geometry are [_touching_][DE-9IM];
    /// `self` and `other` have at least one point in common, but their interiors do not intersect.
    ///
    /// # Notes
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `false`. Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_Touches`][OGR_G_Touches]
    ///
    /// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates
    /// [OGR_G_Touches]: https://gdal.org/api/ogrgeometry_cpp.html#_CPPv4NK11OGRGeometry7TouchesEPK11OGRGeometry
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn touches(&self, other: &Self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_Touches(self.as_ptr(), other.as_ptr()) };
        p != 0
    }

    /// Tests if this geometry and the other geometry are [_crossing_][DE-9IM];
    /// `self` and `other` have some but not all interior points in common, and the dimension of
    /// the intersection is less than that of at least one of them.
    ///
    /// # Notes
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `false`. Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_Crosses`][OGR_G_Crosses]
    ///
    /// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates
    /// [OGR_G_Crosses]: https://gdal.org/api/ogrgeometry_cpp.html#_CPPv4NK11OGRGeometry7CrossesEPK11OGRGeometry
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn crosses(&self, other: &Self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_Crosses(self.as_ptr(), other.as_ptr()) };
        p != 0
    }

    /// Tests if this geometry is [_within_][DE-9IM] the other;
    /// `self` lies fully in the interior of `other`.
    ///
    /// # Notes
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `false`. Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_Within`][OGR_G_Within]
    ///
    /// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates
    /// [OGR_G_Within]: https://gdal.org/api/ogrgeometry_cpp.html#_CPPv4NK11OGRGeometry6WithinEPK11OGRGeometry
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn within(&self, other: &Self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_Within(self.as_ptr(), other.as_ptr()) };
        p != 0
    }

    /// Tests if this geometry and the other [_overlap_][DE-9IM];
    /// `self` and `other` they have some but not all points in common,
    /// they have the same dimension,
    /// and the intersection of the interiors has the same dimension as the geometries.
    ///
    /// # Notes
    /// * Geometry validity is not checked, and invalid geometry will generate unpredictable results.
    /// Use [`Geometry::is_valid`] if validity might be in question.
    /// * If GEOS is *not* enabled, this function will always return `false`. Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`OGR_G_Overlaps`][OGR_G_Overlaps]
    ///
    /// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates
    /// [OGR_G_Overlaps]: https://gdal.org/api/ogrgeometry_cpp.html#_CPPv4NK11OGRGeometry8OverlapsEPK11OGRGeometry
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn overlaps(&self, other: &Self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_Overlaps(self.as_ptr(), other.as_ptr()) };
        p != 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_intersects() {
        let poly = Geometry::from_wkt("POLYGON((0 0,5 5,10 0,0 0))").unwrap();
        let point = Geometry::from_wkt("POINT(10 0)").unwrap();
        assert!(poly.intersects(&point));
    }

    #[test]
    fn test_contains() {
        let poly = Geometry::from_wkt("POLYGON((0 0,5 5,10 0,0 0))").unwrap();
        let point = Geometry::from_wkt("POINT(10 0)").unwrap();
        assert!(!poly.contains(&point));
        let point = Geometry::from_wkt("POINT(0.1 0.01)").unwrap();
        assert!(poly.contains(&point));
    }

    #[test]
    fn test_disjoint() {
        let poly = Geometry::from_wkt("POLYGON((0 0,5 5,10 0,0 0))").unwrap();
        let line = Geometry::from_wkt("LINESTRING(-1 -1, -2 -2)").unwrap();
        assert!(poly.disjoint(&line));
    }

    #[test]
    fn test_touches() {
        let line1 = Geometry::from_wkt("LINESTRING(0 0, 10 10)").unwrap();
        let line2 = Geometry::from_wkt("LINESTRING(0 0, 0 10)").unwrap();
        assert!(line1.touches(&line2));
    }

    #[test]
    fn test_crosses() {
        let line1 = Geometry::from_wkt("LINESTRING(0 0, 10 10)").unwrap();
        let line2 = Geometry::from_wkt("LINESTRING(10 0, 0 10)").unwrap();
        assert!(line1.crosses(&line2));
    }

    #[test]
    fn test_within() {
        let poly1 = Geometry::from_wkt("POLYGON((0 0, 10 10, 10 0, 0 0))").unwrap();
        let poly2 = Geometry::from_wkt("POLYGON((-90 -90, -90 90, 190 -90, -90 -90))").unwrap();
        assert!(poly1.within(&poly2));
        assert!(!poly2.within(&poly1));
    }

    #[test]
    fn test_overlaps() {
        let poly1 = Geometry::from_wkt("POLYGON((0 0, 10 10, 10 0, 0 0))").unwrap();
        let poly2 = Geometry::from_wkt("POLYGON((0 -5,10 5,10 -5,0 -5))").unwrap();
        assert!(poly1.overlaps(&poly2));
    }
}
