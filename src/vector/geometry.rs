use std::{
    cell::RefCell,
    ffi::{c_double, c_int},
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use gdal_sys::{OGRErr, OGRGeometryH, OGRwkbGeometryType};

use crate::errors::*;
use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_null_pointer_err, _string};
use crate::vector::{Envelope, Envelope3D};

/// OGR Geometry
pub struct Geometry {
    c_geometry_ref: RefCell<Option<OGRGeometryH>>,
    owned: bool,
}

impl Geometry {
    /// Create a new Geometry without a C pointer
    ///
    /// # Safety
    /// This method returns a Geometry without wrapped pointer
    pub unsafe fn lazy_feature_geometry() -> Geometry {
        // Geometry objects created with this method map to a Feature's
        // geometry whose memory is managed by the GDAL feature.
        // This object has a tricky lifecycle:
        //
        // * Initially it's created with a null c_geometry
        // * The first time `Feature::geometry` is called, it gets
        //   c_geometry from GDAL and calls `set_c_geometry` with it.
        // * When the Feature is destroyed, this object is also destroyed,
        //   which is good, because that's when c_geometry (which is managed
        //   by the GDAL feature) becomes invalid. Because `self.owned` is
        //   `true`, we don't call `OGR_G_DestroyGeometry`.
        Geometry {
            c_geometry_ref: RefCell::new(None),
            owned: false,
        }
    }

    pub fn has_gdal_ptr(&self) -> bool {
        self.c_geometry_ref.borrow().is_some()
    }

    /// Set the wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn set_c_geometry(&self, c_geometry: OGRGeometryH) {
        assert!(!self.has_gdal_ptr());
        assert!(!self.owned);
        *(self.c_geometry_ref.borrow_mut()) = Some(c_geometry);
    }

    pub(crate) unsafe fn with_c_geometry(c_geom: OGRGeometryH, owned: bool) -> Geometry {
        Geometry {
            c_geometry_ref: RefCell::new(Some(c_geom)),
            owned,
        }
    }

    pub fn empty(wkb_type: OGRwkbGeometryType::Type) -> Result<Geometry> {
        let c_geom = unsafe { gdal_sys::OGR_G_CreateGeometry(wkb_type) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_CreateGeometry"));
        };
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    pub fn is_empty(&self) -> bool {
        unsafe { gdal_sys::OGR_G_IsEmpty(self.c_geometry()) == 1 }
    }

    /// Create a rectangular geometry from West, South, East and North values.
    pub fn bbox(w: f64, s: f64, e: f64, n: f64) -> Result<Geometry> {
        Geometry::from_wkt(&format!(
            "POLYGON (({w} {n}, {e} {n}, {e} {s}, {w} {s}, {w} {n}))",
        ))
    }

    /// Returns a C pointer to the wrapped Geometry
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_geometry(&self) -> OGRGeometryH {
        self.c_geometry_ref.borrow().unwrap()
    }

    /// Returns the C pointer of a Geometry
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn into_c_geometry(mut self) -> OGRGeometryH {
        assert!(self.owned);
        self.owned = false;
        self.c_geometry()
    }

    pub fn set_point(&mut self, i: usize, point: (f64, f64, f64)) {
        let (x, y, z) = point;
        unsafe {
            gdal_sys::OGR_G_SetPoint(
                self.c_geometry(),
                i as c_int,
                x as c_double,
                y as c_double,
                z as c_double,
            );
        };
    }

    pub fn set_point_zm(&mut self, i: usize, point: (f64, f64, f64, f64)) {
        let (x, y, z, m) = point;
        unsafe {
            gdal_sys::OGR_G_SetPointZM(
                self.c_geometry(),
                i as c_int,
                x as c_double,
                y as c_double,
                z as c_double,
                m as c_double,
            );
        };
    }

    pub fn set_point_m(&mut self, i: usize, point: (f64, f64, f64)) {
        let (x, y, m) = point;
        unsafe {
            gdal_sys::OGR_G_SetPointM(
                self.c_geometry(),
                i as c_int,
                x as c_double,
                y as c_double,
                m as c_double,
            );
        };
    }

    pub fn set_point_2d(&mut self, i: usize, p: (f64, f64)) {
        let (x, y) = p;
        unsafe {
            gdal_sys::OGR_G_SetPoint_2D(self.c_geometry(), i as c_int, x as c_double, y as c_double)
        };
    }

    pub fn add_point(&mut self, p: (f64, f64, f64)) {
        let (x, y, z) = p;
        unsafe {
            gdal_sys::OGR_G_AddPoint(
                self.c_geometry(),
                x as c_double,
                y as c_double,
                z as c_double,
            )
        };
    }

    pub fn add_point_2d(&mut self, p: (f64, f64)) {
        let (x, y) = p;
        unsafe { gdal_sys::OGR_G_AddPoint_2D(self.c_geometry(), x as c_double, y as c_double) };
    }

    pub fn add_point_zm(&mut self, p: (f64, f64, f64, f64)) {
        let (x, y, z, m) = p;
        unsafe {
            gdal_sys::OGR_G_AddPointZM(
                self.c_geometry(),
                x as c_double,
                y as c_double,
                z as c_double,
                m as c_double,
            )
        };
    }

    pub fn add_point_m(&mut self, p: (f64, f64, f64)) {
        let (x, y, m) = p;
        unsafe {
            gdal_sys::OGR_G_AddPointM(
                self.c_geometry(),
                x as c_double,
                y as c_double,
                m as c_double,
            )
        };
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
        unsafe { gdal_sys::OGR_G_GetPoint(self.c_geometry(), index, &mut x, &mut y, &mut z) };
        (x, y, z)
    }

    /// Get point coordinates from a line string or a point geometry.
    ///
    /// `index` is the line string vertex index, from 0 to `point_count()-1`, or `0` when a point.
    ///
    /// See: [`OGR_G_GetPointZM`](https://gdal.org/en/stable/api/vector_c_api.html#_CPPv416OGR_G_GetPointZM12OGRGeometryHiPdPdPdPd)
    pub fn get_point_zm(&self, index: i32) -> (f64, f64, f64, f64) {
        let mut x: c_double = 0.;
        let mut y: c_double = 0.;
        let mut z: c_double = 0.;
        let mut m: c_double = 0.;
        unsafe {
            gdal_sys::OGR_G_GetPointZM(self.c_geometry(), index, &mut x, &mut y, &mut z, &mut m)
        };
        (x, y, z, m)
    }

    /// Appends all points in the geometry to `out_points`, as XYZ.
    ///
    /// For some geometry types, like polygons, that don't consist of points, `out_points` will not be modified.
    pub fn get_points(&self, out_points: &mut Vec<(f64, f64, f64)>) -> usize {
        // Consider replacing logic with
        // [OGR_G_GetPoints](https://gdal.org/en/stable/api/vector_c_api.html#_CPPv415OGR_G_GetPoints12OGRGeometryHPviPviPvi)
        let length = unsafe { gdal_sys::OGR_G_GetPointCount(self.c_geometry()) };
        out_points.extend((0..length).map(|i| self.get_point(i)));
        length as usize
    }

    /// Appends all points in the geometry to `out_points`, as XYZM.
    ///
    /// For some geometry types, like polygons, that don't consist of points, `out_points` will not be modified.
    pub fn get_points_zm(&self, out_points: &mut Vec<(f64, f64, f64, f64)>) -> usize {
        // Consider replacing logic with
        // [OGR_G_GetPoints](https://gdal.org/en/stable/api/vector_c_api.html#_CPPv415OGR_G_GetPoints12OGRGeometryHPviPviPvi)
        let length = unsafe { gdal_sys::OGR_G_GetPointCount(self.c_geometry()) };
        out_points.extend((0..length).map(|i| self.get_point_zm(i)));
        length as usize
    }

    /// Get the geometry type ordinal
    ///
    /// See: [OGR_G_GetGeometryType](https://gdal.org/api/vector_c_api.html#_CPPv421OGR_G_GetGeometryType12OGRGeometryH)
    pub fn geometry_type(&self) -> OGRwkbGeometryType::Type {
        unsafe { gdal_sys::OGR_G_GetGeometryType(self.c_geometry()) }
    }

    /// Get the WKT name for the type of this geometry.
    ///
    /// See: [`OGR_G_GetGeometryName`](https://gdal.org/api/vector_c_api.html#_CPPv421OGR_G_GetGeometryName12OGRGeometryH)
    pub fn geometry_name(&self) -> String {
        // Note: C API makes no statements about this possibly returning null.
        // So we don't have to result wrap this,
        let c_str = unsafe { gdal_sys::OGR_G_GetGeometryName(self.c_geometry()) };
        _string(c_str).unwrap_or_default()
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
        let cnt = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) };
        cnt as usize
    }

    /// Get the number of points from a Point or a LineString/LinearRing geometry.
    ///
    /// Only `wkbPoint` or `wkbLineString` may return a non-zero value. Other geometry types will return 0.
    ///
    /// See: [`OGR_G_GetPointCount`](https://gdal.org/api/vector_c_api.html#_CPPv419OGR_G_GetPointCount12OGRGeometryH)
    pub fn point_count(&self) -> usize {
        let cnt = unsafe { gdal_sys::OGR_G_GetPointCount(self.c_geometry()) };
        cnt as usize
    }

    /// Returns the n-th sub-geometry as a non-owned Geometry.
    ///
    /// # Safety
    /// Don't keep this object for long.
    pub unsafe fn get_unowned_geometry(&self, n: usize) -> Geometry {
        // get the n-th sub-geometry as a non-owned Geometry; don't keep this
        // object for long.
        let c_geom = gdal_sys::OGR_G_GetGeometryRef(self.c_geometry(), n as c_int);
        Geometry::with_c_geometry(c_geom, false)
    }

    /// Get a reference to the geometry at given `index`
    pub fn get_geometry(&self, index: usize) -> GeometryRef<'_> {
        let geom = unsafe { self.get_unowned_geometry(index) };
        GeometryRef {
            geom,
            _lifetime: PhantomData,
        }
    }

    pub fn add_geometry(&mut self, mut sub: Geometry) -> Result<()> {
        assert!(sub.owned);
        sub.owned = false;
        let rv =
            unsafe { gdal_sys::OGR_G_AddGeometryDirectly(self.c_geometry(), sub.c_geometry()) };
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
        unsafe { gdal_sys::OGR_G_Length(self.c_geometry()) }
    }

    /// Compute geometry area in square units of the spatial reference system in use.
    ///
    /// Supported for `LinearRing`, `Polygon` and `MultiPolygon`.
    /// Returns zero for all other geometry types.
    ///
    /// See: [`OGR_G_Area`](https://gdal.org/api/vector_c_api.html#_CPPv410OGR_G_Area12OGRGeometryH)
    pub fn area(&self) -> f64 {
        unsafe { gdal_sys::OGR_G_Area(self.c_geometry()) }
    }

    /// Computes and returns the axis-aligned 2D bounding envelope for this geometry.
    ///
    /// See: [`OGR_G_GetEnvelope`](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_GetEnvelope12OGRGeometryHP11OGREnvelope)
    pub fn envelope(&self) -> Envelope {
        let mut envelope = MaybeUninit::uninit();
        unsafe {
            gdal_sys::OGR_G_GetEnvelope(self.c_geometry(), envelope.as_mut_ptr());
            envelope.assume_init()
        }
    }

    /// Computes and returns the axis aligned 3D bounding envelope for this geometry.
    ///
    /// See: [`OGR_G_GetEnvelope3D`](https://gdal.org/api/vector_c_api.html#_CPPv419OGR_G_GetEnvelope3D12OGRGeometryHP13OGREnvelope3D)
    pub fn envelope_3d(&self) -> Envelope3D {
        let mut envelope = MaybeUninit::uninit();
        unsafe {
            gdal_sys::OGR_G_GetEnvelope3D(self.c_geometry(), envelope.as_mut_ptr());
            envelope.assume_init()
        }
    }

    /// Converts geometry to 2D.
    ///
    /// See: [`OGR_G_FlattenTo2D`](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_FlattenTo2D12OGRGeometryH)
    pub fn flatten_to_2d(&mut self) {
        unsafe { gdal_sys::OGR_G_FlattenTo2D(self.c_geometry()) };
    }

    /// Get the spatial reference system for this geometry.
    ///
    /// Returns `Some(SpatialRef)`, or `None` if one isn't defined.
    ///
    /// See: [OGR_G_GetSpatialReference](https://gdal.org/doxygen/ogr__api_8h.html#abc393e40282eec3801fb4a4abc9e25bf)
    pub fn spatial_ref(&self) -> Option<SpatialRef> {
        let c_spatial_ref = unsafe { gdal_sys::OGR_G_GetSpatialReference(self.c_geometry()) };

        if c_spatial_ref.is_null() {
            None
        } else {
            unsafe { SpatialRef::from_c_obj(c_spatial_ref) }.ok()
        }
    }

    pub fn set_spatial_ref(&mut self, spatial_ref: SpatialRef) {
        unsafe {
            gdal_sys::OGR_G_AssignSpatialReference(self.c_geometry(), spatial_ref.to_c_hsrs())
        };
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
        let p = unsafe { gdal_sys::OGR_G_IsValid(self.c_geometry()) };
        p != 0
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        if self.owned {
            let c_geometry = self.c_geometry_ref.borrow();
            unsafe { gdal_sys::OGR_G_DestroyGeometry(c_geometry.unwrap()) };
        }
    }
}

impl Clone for Geometry {
    fn clone(&self) -> Geometry {
        // assert!(self.has_gdal_ptr());
        let c_geometry = self.c_geometry_ref.borrow();
        let new_c_geom = unsafe { gdal_sys::OGR_G_Clone(c_geometry.unwrap()) };
        unsafe { Geometry::with_c_geometry(new_c_geom, true) }
    }
}

impl Debug for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wkt = self.iso_wkt();

        match wkt {
            Ok(wkt) => f.write_str(wkt.as_str()),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl PartialEq for Geometry {
    fn eq(&self, other: &Self) -> bool {
        unsafe { gdal_sys::OGR_G_Equals(self.c_geometry(), other.c_geometry()) != 0 }
    }
}

impl Eq for Geometry {}

pub fn geometry_type_to_name(ty: OGRwkbGeometryType::Type) -> String {
    let rv = unsafe { gdal_sys::OGRGeometryTypeToName(ty) };
    _string(rv).unwrap_or_default()
}

/// Returns the 2D geometry type corresponding to the passed geometry type.
pub fn geometry_type_flatten(ty: OGRwkbGeometryType::Type) -> OGRwkbGeometryType::Type {
    unsafe { gdal_sys::OGR_GT_Flatten(ty) }
}

/// Returns the 3D geometry type corresponding to the passed geometry type.
pub fn geometry_type_set_z(ty: OGRwkbGeometryType::Type) -> OGRwkbGeometryType::Type {
    unsafe { gdal_sys::OGR_GT_SetZ(ty) }
}

/// Returns the measured geometry type corresponding to the passed geometry type.
pub fn geometry_type_set_m(ty: OGRwkbGeometryType::Type) -> OGRwkbGeometryType::Type {
    unsafe { gdal_sys::OGR_GT_SetM(ty) }
}

/// Returns a XY, XYZ, XYM or XYZM geometry type depending on parameter.
pub fn geometry_type_set_modifier(
    ty: OGRwkbGeometryType::Type,
    set_z: bool,
    set_m: bool,
) -> OGRwkbGeometryType::Type {
    unsafe { gdal_sys::OGR_GT_SetModifier(ty, set_z as i32, set_m as i32) }
}

/// Returns `true` if the geometry type is a 3D geometry type.
pub fn geometry_type_has_z(ty: OGRwkbGeometryType::Type) -> bool {
    unsafe { gdal_sys::OGR_GT_HasZ(ty) != 0 }
}

/// Returns `true` if the geometry type is a measured type.
pub fn geometry_type_has_m(ty: OGRwkbGeometryType::Type) -> bool {
    unsafe { gdal_sys::OGR_GT_HasM(ty) != 0 }
}

/// Reference to owned geometry
pub struct GeometryRef<'a> {
    geom: Geometry,
    _lifetime: PhantomData<&'a ()>,
}

impl Deref for GeometryRef<'_> {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.geom
    }
}

impl DerefMut for GeometryRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geom
    }
}

impl Debug for GeometryRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.geom, f)
    }
}

#[cfg(test)]
mod tests {
    use self::OGRwkbGeometryType::{wkbLineStringZM, wkbMultiPointZM, wkbPointZM};

    use super::*;
    use crate::spatial_ref::SpatialRef;
    use crate::test_utils::SuppressGDALErrorLog;
    use gdal_sys::OGRwkbGeometryType::{
        wkbLineString, wkbLinearRing, wkbMultiPoint, wkbMultiPolygon, wkbPoint, wkbPolygon,
    };

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
    pub fn test_create_multipoint_zm() {
        let mut geom = Geometry::empty(wkbMultiPointZM).unwrap();
        let mut point = Geometry::empty(wkbPointZM).unwrap();
        point.add_point_zm((1.0, 2.0, 3.0, 0.0));
        geom.add_geometry(point).unwrap();
        let mut point = Geometry::empty(wkbPointZM).unwrap();
        point.add_point_zm((3.0, 2.0, 1.0, 1.0));
        assert!(!point.is_empty());
        point.set_point_zm(0, (4.0, 2.0, 1.0, 1.0));
        geom.add_geometry(point).unwrap();
        assert!(!geom.is_empty());
        let expected =
            Geometry::from_wkt("MULTIPOINT ZM ((1.0 2.0 3.0 0.0), (4.0 2.0 1.0 1.0))").unwrap();
        assert_eq!(geom, expected);
        assert_eq!(
            geometry_type_has_m(geom.geometry_type()),
            geometry_type_has_m(expected.geometry_type())
        )
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
        let mut ring_vec: Vec<(f64, f64, f64)> = Vec::new();
        ring.get_points(&mut ring_vec);
        assert_eq!(ring_vec.len(), 6);
        let mut poly = Geometry::empty(wkbPolygon).unwrap();
        poly.add_geometry(ring.to_owned()).unwrap();
        let mut poly_vec: Vec<(f64, f64, f64)> = Vec::new();
        poly.get_points(&mut poly_vec);
        // Points are in ring, not containing geometry.
        // NB: In Python SWIG bindings, `GetPoints` is fallible.
        assert!(poly_vec.is_empty());
        assert_eq!(poly.geometry_count(), 1);
        let ring_out = poly.get_geometry(0);
        let mut ring_out_vec: Vec<(f64, f64, f64)> = Vec::new();
        ring_out.get_points(&mut ring_out_vec);
        // NB: `wkb()` shows it to be a `LINEARRING`, but returned type is LineString
        assert_eq!(ring_out.geometry_type(), wkbLineString);
        assert!(!&ring_out.is_empty());
        assert_eq!(ring_vec, ring_out_vec);
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
        let mut points: Vec<(f64, f64, f64)> = Vec::new();
        inner.get_points(&mut points);
        assert!(!points.is_empty());
    }

    #[test]
    fn test_get_points_zm() {
        let mut line = Geometry::empty(wkbLineStringZM).unwrap();
        line.add_point_zm((0.0, 0.0, 0.0, 0.0));
        line.add_point_zm((1.0, 0.0, 0.25, 0.5));
        line.add_point_zm((1.0, 1.0, 0.5, 1.0));
        let mut line_points: Vec<(f64, f64, f64, f64)> = Vec::new();
        line.get_points_zm(&mut line_points);
        assert_eq!(line_points.len(), 3);
        assert_eq!(line_points.get(2), Some(&(1.0, 1.0, 0.5, 1.0)));
    }

    #[test]
    pub fn test_geometry_type_to_name() {
        assert_eq!(geometry_type_to_name(wkbLineString), "Line String");
        // We don't care what it returns when passed an invalid value, just that it doesn't crash.
        geometry_type_to_name(4372521);
    }

    #[test]
    pub fn test_geometry_modify() {
        let polygon = Geometry::from_wkt("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))").unwrap();
        for i in 0..polygon.geometry_count() {
            let mut ring = polygon.get_geometry(i);
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

    #[test]
    fn test_geometry_type_modification() {
        let mut geom_type = OGRwkbGeometryType::wkbPoint;
        geom_type = geometry_type_set_z(geom_type);
        assert_eq!(geom_type, OGRwkbGeometryType::wkbPoint25D);
        geom_type = geometry_type_set_m(geom_type);
        assert_eq!(geom_type, OGRwkbGeometryType::wkbPointZM);
        geom_type = geometry_type_set_modifier(geom_type, false, true);
        assert_eq!(geom_type, OGRwkbGeometryType::wkbPointM);
        geom_type = geometry_type_flatten(geom_type);
        assert_eq!(geom_type, OGRwkbGeometryType::wkbPoint);
    }

    #[test]
    fn test_geometry_type_has_zm() {
        let geom = Geometry::from_wkt("POINT(0 1)").unwrap();
        assert!(!geometry_type_has_z(geom.geometry_type()));
        assert!(!geometry_type_has_m(geom.geometry_type()));
        let geom = Geometry::from_wkt("POINT(0 1 2)").unwrap();
        assert!(geometry_type_has_z(geom.geometry_type()));
        assert!(!geometry_type_has_m(geom.geometry_type()));
        let geom = Geometry::from_wkt("POINT ZM (0 1 2 3)").unwrap();
        assert!(geometry_type_has_z(geom.geometry_type()));
        assert!(geometry_type_has_m(geom.geometry_type()));
    }
}
