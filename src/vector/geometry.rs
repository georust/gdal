use std::fmt::{self, Debug};
use std::mem::{ManuallyDrop, MaybeUninit};

use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};
use libc::{c_double, c_int};

use gdal_sys::{self, OGRErr, OGRwkbGeometryType};

use crate::errors::*;
use crate::spatial_ref::{SpatialRef, SpatialRefRef};
use crate::utils::{_last_null_pointer_err, _string};
use crate::vector::{Envelope, Envelope3D};

foreign_type! {
    /// OGR Geometry
    pub unsafe type Geometry {
        type CType = libc::c_void;
        fn drop = gdal_sys::OGR_G_DestroyGeometry;
        fn clone = gdal_sys::OGR_G_Clone;
    }
}

impl Geometry {
    pub fn empty(wkb_type: OGRwkbGeometryType::Type) -> Result<Geometry> {
        let c_geom = unsafe { gdal_sys::OGR_G_CreateGeometry(wkb_type) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_CreateGeometry"));
        };
        Ok(unsafe { Geometry::from_ptr(c_geom) })
    }

    /// Create a rectangular geometry from West, South, East and North values.
    pub fn bbox(w: f64, s: f64, e: f64, n: f64) -> Result<Geometry> {
        Geometry::from_wkt(&format!(
            "POLYGON (({w} {n}, {e} {n}, {e} {s}, {w} {s}, {w} {n}))",
        ))
    }
}

impl GeometryRef {
    pub fn is_empty(&self) -> bool {
        unsafe { gdal_sys::OGR_G_IsEmpty(self.as_ptr()) == 1 }
    }

    pub fn set_point(&mut self, i: usize, point: (f64, f64, f64)) {
        let (x, y, z) = point;
        unsafe {
            gdal_sys::OGR_G_SetPoint(
                self.as_ptr(),
                i as c_int,
                x as c_double,
                y as c_double,
                z as c_double,
            );
        };
    }

    pub fn set_point_2d(&mut self, i: usize, p: (f64, f64)) {
        let (x, y) = p;
        unsafe {
            gdal_sys::OGR_G_SetPoint_2D(self.as_ptr(), i as c_int, x as c_double, y as c_double)
        };
    }

    pub fn add_point(&mut self, p: (f64, f64, f64)) {
        let (x, y, z) = p;
        unsafe {
            gdal_sys::OGR_G_AddPoint(self.as_ptr(), x as c_double, y as c_double, z as c_double)
        };
    }

    pub fn add_point_2d(&mut self, p: (f64, f64)) {
        let (x, y) = p;
        unsafe { gdal_sys::OGR_G_AddPoint_2D(self.as_ptr(), x as c_double, y as c_double) };
    }

    /// Get point coordinates from a line string or a point geometry.
    ///
    /// `index` is the line string vertex index, from 0 to `point_count()-1`, or `0` when a point.
    ///
    /// See: [`OGR_G_GetPoint`](https://gdal.org/api/vector_c_api.html#_CPPv414OGR_G_GetPoint12OGRGeometryHiPdPdPd)
    pub fn get_point(&self, index: i32) -> (f64, f64, f64) {
        let mut x: c_double = 0.;
        let mut y: c_double = 0.;
        let mut z: c_double = 0.;
        unsafe { gdal_sys::OGR_G_GetPoint(self.as_ptr(), index, &mut x, &mut y, &mut z) };
        (x, y, z)
    }

    pub fn get_point_vec(&self) -> Vec<(f64, f64, f64)> {
        let length = unsafe { gdal_sys::OGR_G_GetPointCount(self.as_ptr()) };
        (0..length).map(|i| self.get_point(i)).collect()
    }

    /// Get the geometry type ordinal
    ///
    /// See: [OGR_G_GetGeometryType](https://gdal.org/api/vector_c_api.html#_CPPv421OGR_G_GetGeometryType12OGRGeometryH)
    pub fn geometry_type(&self) -> OGRwkbGeometryType::Type {
        unsafe { gdal_sys::OGR_G_GetGeometryType(self.as_ptr()) }
    }

    /// Get the WKT name for the type of this geometry.
    ///
    /// See: [`OGR_G_GetGeometryName`](https://gdal.org/api/vector_c_api.html#_CPPv421OGR_G_GetGeometryName12OGRGeometryH)
    pub fn geometry_name(&self) -> String {
        // Note: C API makes no statements about this possibly returning null.
        // So we don't have to result wrap this,
        let c_str = unsafe { gdal_sys::OGR_G_GetGeometryName(self.as_ptr()) };
        if c_str.is_null() {
            "".into()
        } else {
            _string(c_str)
        }
    }

    /// Get the number of elements in a geometry, or number of geometries in container.
    ///
    /// Only geometries of type `wkbPolygon`, `wkbMultiPoint`, `wkbMultiLineString`, `wkbMultiPolygon`
    /// or `wkbGeometryCollection` may return a non-zero value. Other geometry types will return 0.
    ///
    /// For a polygon, the returned number is the number of rings (exterior ring + interior rings).
    ///
    /// See: [`OGR_G_GetGeometryCount`](https://gdal.org/api/vector_c_api.html#_CPPv422OGR_G_GetGeometryCount12OGRGeometryH)
    pub fn geometry_count(&self) -> usize {
        let cnt = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.as_ptr()) };
        cnt as usize
    }

    /// Get the number of points from a Point or a LineString/LinearRing geometry.
    ///
    /// Only `wkbPoint` or `wkbLineString` may return a non-zero value. Other geometry types will return 0.
    ///
    /// See: [`OGR_G_GetPointCount`](https://gdal.org/api/vector_c_api.html#_CPPv419OGR_G_GetPointCount12OGRGeometryH)
    pub fn point_count(&self) -> usize {
        let cnt = unsafe { gdal_sys::OGR_G_GetPointCount(self.as_ptr()) };
        cnt as usize
    }

    /// Get a reference to the geometry at given `index`
    ///
    /// # Arguments
    /// * `index`: the index of the geometry to fetch, between 0 and getNumGeometries() - 1.
    pub fn get_geometry(&self, index: usize) -> &GeometryRef {
        let c_geom = unsafe { gdal_sys::OGR_G_GetGeometryRef(self.as_ptr(), index as c_int) };
        unsafe { GeometryRef::from_ptr(c_geom) }
    }

    /// Get a mutable reference to the geometry at given `index`
    ///
    /// # Arguments
    /// * `index`: the index of the geometry to fetch, between 0 and getNumGeometries() - 1.
    pub fn get_geometry_mut(&mut self, index: usize) -> &mut GeometryRef {
        let c_geom = unsafe { gdal_sys::OGR_G_GetGeometryRef(self.as_ptr(), index as c_int) };
        unsafe { GeometryRef::from_ptr_mut(c_geom) }
    }

    /// Add a subgeometry
    ///
    /// # Arguments
    /// * `sub`: geometry to add as a child of `self`.
    pub fn add_geometry(&mut self, sub: Geometry) -> Result<()> {
        // `OGR_G_AddGeometryDirectly` takes ownership of the Geometry.
        // According to https://doc.rust-lang.org/std/mem/fn.forget.html#relationship-with-manuallydrop
        // `ManuallyDrop` is the suggested means of transferring memory management while
        // robustly preventing a double-free.
        let sub = ManuallyDrop::new(sub);

        let rv = unsafe { gdal_sys::OGR_G_AddGeometryDirectly(self.as_ptr(), sub.as_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_AddGeometryDirectly",
            });
        }
        Ok(())
    }

    /// Compute geometry area in units of the spatial reference system in use.
    ///
    /// Supported for `Curve` (including `LineString` and `CircularString`) and `MultiCurve`.
    /// Returns zero for all other geometry types.
    ///
    /// See: [`OGR_G_Length`](https://gdal.org/api/vector_c_api.html#_CPPv412OGR_G_Length12OGRGeometryH)
    pub fn length(&self) -> f64 {
        unsafe { gdal_sys::OGR_G_Length(self.as_ptr()) }
    }

    /// Compute geometry area in square units of the spatial reference system in use.
    ///
    /// Supported for `LinearRing`, `Polygon` and `MultiPolygon`.
    /// Returns zero for all other geometry types.
    ///
    /// See: [`OGR_G_Area`](https://gdal.org/api/vector_c_api.html#_CPPv410OGR_G_Area12OGRGeometryH)
    pub fn area(&self) -> f64 {
        unsafe { gdal_sys::OGR_G_Area(self.as_ptr()) }
    }

    /// Computes and returns the axis-aligned 2D bounding envelope for this geometry.
    ///
    /// See: [`OGR_G_GetEnvelope`](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_GetEnvelope12OGRGeometryHP11OGREnvelope)
    pub fn envelope(&self) -> Envelope {
        let mut envelope = MaybeUninit::uninit();
        unsafe {
            gdal_sys::OGR_G_GetEnvelope(self.as_ptr(), envelope.as_mut_ptr());
            envelope.assume_init()
        }
    }

    /// Computes and returns the axis aligned 3D bounding envelope for this geometry.
    ///
    /// See: [`OGR_G_GetEnvelope3D`](https://gdal.org/api/vector_c_api.html#_CPPv419OGR_G_GetEnvelope3D12OGRGeometryHP13OGREnvelope3D)
    pub fn envelope_3d(&self) -> Envelope3D {
        let mut envelope = MaybeUninit::uninit();
        unsafe {
            gdal_sys::OGR_G_GetEnvelope3D(self.as_ptr(), envelope.as_mut_ptr());
            envelope.assume_init()
        }
    }

    /// Converts geometry to 2D.
    ///
    /// See: [`OGR_G_FlattenTo2D`](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_FlattenTo2D12OGRGeometryH)
    pub fn flatten_to_2d(&mut self) {
        unsafe { gdal_sys::OGR_G_FlattenTo2D(self.as_ptr()) };
    }

    /// Get the spatial reference system for this geometry.
    ///
    /// Returns `Some(SpatialRef)`, or `None` if one isn't defined.
    ///
    /// See: [OGR_G_GetSpatialReference](https://gdal.org/doxygen/ogr__api_8h.html#abc393e40282eec3801fb4a4abc9e25bf)
    pub fn spatial_ref(&self) -> Option<SpatialRef> {
        let c_spatial_ref = unsafe { gdal_sys::OGR_G_GetSpatialReference(self.as_ptr()) };

        if c_spatial_ref.is_null() {
            None
        } else {
            Some(unsafe { SpatialRefRef::from_ptr(c_spatial_ref).to_owned() })
        }
    }

    pub fn set_spatial_ref(&mut self, spatial_ref: SpatialRef) {
        unsafe { gdal_sys::OGR_G_AssignSpatialReference(self.as_ptr(), spatial_ref.as_ptr()) };
    }

    /// Test if the geometry is valid.
    ///
    /// # Notes
    /// This function requires the GEOS library.
    /// If OGR is built without the GEOS library, this function will always return `false`.
    /// Check with [`VersionInfo::has_geos`][has_geos].
    ///
    /// See: [`Self::make_valid`]
    /// See: [`OGR_G_IsValid`](https://gdal.org/api/vector_c_api.html#_CPPv413OGR_G_IsValid12OGRGeometryH)
    ///
    /// [has_geos]: crate::version::VersionInfo::has_geos
    pub fn is_valid(&self) -> bool {
        let p = unsafe { gdal_sys::OGR_G_IsValid(self.as_ptr()) };
        p != 0
    }
}

impl Debug for GeometryRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.wkt() {
            Ok(wkt) => f.write_str(wkt.as_str()),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl PartialEq for GeometryRef {
    fn eq(&self, other: &Self) -> bool {
        unsafe { gdal_sys::OGR_G_Equal(self.as_ptr(), other.as_ptr()) != 0 }
    }
}

impl Eq for GeometryRef {}

impl Debug for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_ref(), f)
    }
}

impl PartialEq for Geometry {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_ref(), other.as_ref())
    }
}

impl Eq for Geometry {}

pub fn geometry_type_to_name(ty: OGRwkbGeometryType::Type) -> String {
    let rv = unsafe { gdal_sys::OGRGeometryTypeToName(ty) };
    // If the type is invalid, OGRGeometryTypeToName returns a valid string anyway.
    assert!(!rv.is_null());
    _string(rv)
}

#[cfg(test)]
mod tests {
    use gdal_sys::OGRwkbGeometryType::{
        wkbLineString, wkbLinearRing, wkbMultiPoint, wkbMultiPolygon, wkbPoint, wkbPolygon,
    };

    use crate::spatial_ref::SpatialRef;
    use crate::test_utils::SuppressGDALErrorLog;

    use super::*;

    #[test]
    fn test_create_bbox() {
        let bbox = Geometry::bbox(-27., 33., 52., 85.).unwrap();
        assert_eq!(bbox.json().unwrap(), "{ \"type\": \"Polygon\", \"coordinates\": [ [ [ -27.0, 85.0 ], [ 52.0, 85.0 ], [ 52.0, 33.0 ], [ -27.0, 33.0 ], [ -27.0, 85.0 ] ] ] }");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    pub fn test_length() {
        let _nolog = SuppressGDALErrorLog::new();
        let geom = Geometry::empty(wkbPoint).unwrap();
        assert_eq!(geom.area(), 0.0);

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert_eq!(geom.area(), 0.0);

        let wkt = "LINESTRING (0 10, 10 10, 10 15)";
        let geom = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geom.length() as i32, 15);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    pub fn test_area() {
        let _nolog = SuppressGDALErrorLog::new();
        let geom = Geometry::empty(wkbMultiPolygon).unwrap();
        assert_eq!(geom.area(), 0.0);

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert_eq!(geom.area(), 0.0);

        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let geom = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geom.area().floor(), 25.0);
    }

    #[test]
    pub fn test_is_empty() {
        let geom = Geometry::empty(wkbMultiPolygon).unwrap();
        assert!(geom.is_empty());

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert!(!geom.is_empty());

        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let geom = Geometry::from_wkt(wkt).unwrap();
        assert!(!geom.is_empty());
    }

    #[test]
    pub fn test_flatten_to_2d() {
        let mut geom = Geometry::from_wkt("POINT (0 1 2)").unwrap();
        geom.flatten_to_2d();
        assert_eq!(geom.wkt().unwrap(), "POINT (0 1)");
    }

    #[test]
    pub fn test_create_multipoint_2d() {
        let mut geom = Geometry::empty(wkbMultiPoint).unwrap();
        let mut point = Geometry::empty(wkbPoint).unwrap();
        point.add_point_2d((1.0, 2.0));
        geom.add_geometry(point).unwrap();
        let mut point = Geometry::empty(wkbPoint).unwrap();
        point.add_point_2d((2.0, 3.0));
        assert!(!point.is_empty());
        point.set_point_2d(0, (2.0, 4.0));
        geom.add_geometry(point).unwrap();
        assert!(!geom.is_empty());

        let expected = Geometry::from_wkt("MULTIPOINT((1.0 2.0), (2.0 4.0))").unwrap();
        assert_eq!(geom, expected);
    }

    #[test]
    pub fn test_create_multipoint_3d() {
        let mut geom = Geometry::empty(wkbMultiPoint).unwrap();
        let mut point = Geometry::empty(wkbPoint).unwrap();
        point.add_point((1.0, 2.0, 3.0));
        geom.add_geometry(point).unwrap();
        let mut point = Geometry::empty(wkbPoint).unwrap();
        point.add_point((3.0, 2.0, 1.0));
        assert!(!point.is_empty());
        point.set_point(0, (4.0, 2.0, 1.0));
        geom.add_geometry(point).unwrap();
        assert!(!geom.is_empty());

        let expected = Geometry::from_wkt("MULTIPOINT((1.0 2.0 3.0), (4.0 2.0 1.0))").unwrap();
        assert_eq!(geom, expected);
    }

    #[test]
    pub fn test_spatial_ref() {
        let geom = Geometry::empty(wkbMultiPolygon).unwrap();
        assert!(geom.spatial_ref().is_none());

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert!(geom.spatial_ref().is_none());

        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let mut geom = Geometry::from_wkt(wkt).unwrap();
        assert!(geom.spatial_ref().is_none());

        let srs = SpatialRef::from_epsg(4326).unwrap();
        geom.set_spatial_ref(srs);
        assert!(geom.spatial_ref().is_some());
    }

    #[test]
    fn test_ring_points() {
        let mut ring = Geometry::empty(wkbLinearRing).unwrap();
        ring.add_point_2d((1179091.1646903288, 712782.8838459781));
        ring.add_point_2d((1161053.0218226474, 667456.2684348812));
        ring.add_point_2d((1214704.933941905, 641092.8288590391));
        ring.add_point_2d((1228580.428455506, 682719.3123998424));
        ring.add_point_2d((1218405.0658121984, 721108.1805541387));
        ring.add_point_2d((1179091.1646903288, 712782.8838459781));
        assert!(!ring.is_empty());
        assert_eq!(ring.get_point_vec().len(), 6);
        let mut poly = Geometry::empty(wkbPolygon).unwrap();
        poly.add_geometry(ring.to_owned()).unwrap();
        // Points are in ring, not containing geometry.
        // NB: In Python SWIG bindings, `GetPoints` is fallible.
        assert!(poly.get_point_vec().is_empty());
        assert_eq!(poly.geometry_count(), 1);
        let ring_out = poly.get_geometry(0);
        // NB: `wkb()` shows it to be a `LINEARRING`, but returned type is LineString
        assert_eq!(ring_out.geometry_type(), wkbLineString);
        assert!(!&ring_out.is_empty());
        assert_eq!(ring.get_point_vec(), ring_out.get_point_vec());
    }

    #[test]
    fn test_get_inner_points() {
        let geom = Geometry::bbox(0., 0., 1., 1.).unwrap();
        assert!(!geom.is_empty());
        assert_eq!(geom.geometry_count(), 1);
        assert!(geom.area() > 0.);
        assert_eq!(geom.geometry_type(), OGRwkbGeometryType::wkbPolygon);
        assert!(geom.json().unwrap().contains("Polygon"));
        let inner = geom.get_geometry(0);
        let points = inner.get_point_vec();
        assert!(!points.is_empty());
    }

    #[test]
    pub fn test_geometry_type_to_name() {
        assert_eq!(geometry_type_to_name(wkbLineString), "Line String");
        // We don't care what it returns when passed an invalid value, just that it doesn't crash.
        geometry_type_to_name(4372521);
    }

    #[test]
    pub fn test_geometry_modify() {
        let mut polygon =
            Geometry::from_wkt("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))").unwrap();
        for i in 0..polygon.geometry_count() {
            let ring = &mut polygon.get_geometry_mut(i);
            for j in 0..ring.point_count() {
                let (x, y, _) = ring.get_point(j as i32);
                ring.set_point_2d(j, (x * 10.0, y * 10.0));
            }
        }
        assert_eq!(
            "POLYGON ((300 100,400 400,200 400,100 200,300 100))",
            polygon.wkt().unwrap()
        );
    }
}
