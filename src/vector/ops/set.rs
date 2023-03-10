use crate::vector::Geometry;

/// # Set Operations
///
/// These methods provide set operations over two geometries, producing a new geometry.
impl Geometry {
    /// Compute intersection.
    ///
    /// Generates a new geometry which is the region of intersection of
    /// the two geometries operated on. Call intersects (Not yet implemented)
    /// to check if there is a region of intersection.
    /// Geometry validity is not checked. In case you are unsure of the
    /// validity of the input geometries, call IsValid() before,
    /// otherwise the result might be wrong.
    ///
    /// # Returns
    /// Some(Geometry) if both Geometries contain pointers
    /// None if either geometry is missing the gdal pointer, or there is an error.
    ///
    /// See: [`OGR_G_Intersection`](https://gdal.org/api/vector_c_api.html#_CPPv418OGR_G_Intersection12OGRGeometryH12OGRGeometryH)
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
        let other =
           Geometry::from_wkt("POLYGON ((1 -5, 1 1, -5 1, -5 -5, 1 -5))").unwrap();

        let inter = geom.union(&other);

        assert!(inter.is_some());

        let inter = inter.unwrap();

        assert_eq!(inter.area(), 135.0);
    }

    #[test]
    fn test_union_no_gdal_ptr() {
        let geom =
           Geometry::from_wkt("POLYGON ((0.0 10.0, 0.0 0.0, 10.0 0.0, 10.0 10.0, 0.0 10.0))")
              .unwrap();
        let other = unsafe { Geometry::lazy_feature_geometry() };

        let inter = geom.union(&other);

        assert!(inter.is_none());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_union_no_intersects() {
        let geom =
           Geometry::from_wkt("POLYGON ((0.0 5.0, 0.0 0.0, 5.0 0.0, 5.0 5.0, 0.0 5.0))").unwrap();

        let other =
           Geometry::from_wkt("POLYGON ((15.0 15.0, 15.0 20.0, 20.0 20.0, 20.0 15.0, 15.0 15.0))")
              .unwrap();

        let inter = geom.union(&other);

        assert!(inter.is_some());

        assert_eq!(inter.unwrap().area(), 50.0);
    }
}
