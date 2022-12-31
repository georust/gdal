use std::cell::RefCell;
use std::ffi::CString;
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::null_mut;

use libc::{c_char, c_double, c_int, c_void};

use crate::cpl::CslStringList;
use gdal_sys::{self, OGRErr, OGRGeometryH, OGRwkbGeometryType};

use crate::errors::*;
use crate::spatial_ref::{CoordTransform, SpatialRef};
use crate::utils::{_last_null_pointer_err, _string};

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

    /// Create a geometry by parsing a
    /// [WKT](https://en.wikipedia.org/wiki/Well-known_text) string.
    pub fn from_wkt(wkt: &str) -> Result<Geometry> {
        let c_wkt = CString::new(wkt)?;
        // OGR_G_CreateFromWkt does not write to the pointed-to memory, but this is not reflected
        // in its signature (`char**` instead of `char const**`), so we need a scary looking cast.
        let mut c_wkt_ptr = c_wkt.as_ptr() as *mut c_char;
        let mut c_geom = null_mut();
        let rv = unsafe { gdal_sys::OGR_G_CreateFromWkt(&mut c_wkt_ptr, null_mut(), &mut c_geom) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_CreateFromWkt",
            });
        }
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Creates a geometry by parsing a slice of bytes in
    /// [WKB](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry#Well-known_binary)
    /// (Well-Known Binary) format.
    pub fn from_wkb(wkb: &[u8]) -> Result<Geometry> {
        let mut c_geom = null_mut();
        let rv = unsafe {
            gdal_sys::OGR_G_CreateFromWkb(
                wkb.as_ptr() as *const std::ffi::c_void,
                null_mut(),
                &mut c_geom,
                wkb.len() as i32,
            )
        };
        if rv != gdal_sys::OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_CreateFromWkb",
            });
        }
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Create a rectangular geometry from West, South, East and North values.
    pub fn bbox(w: f64, s: f64, e: f64, n: f64) -> Result<Geometry> {
        Geometry::from_wkt(&format!(
            "POLYGON (({} {}, {} {}, {} {}, {} {}, {} {}))",
            w, n, e, n, e, s, w, s, w, n,
        ))
    }

    /// Serialize the geometry as JSON.
    pub fn json(&self) -> Result<String> {
        let c_json = unsafe { gdal_sys::OGR_G_ExportToJson(self.c_geometry()) };
        if c_json.is_null() {
            return Err(_last_null_pointer_err("OGR_G_ExportToJson"));
        };
        let rv = _string(c_json);
        unsafe { gdal_sys::VSIFree(c_json as *mut c_void) };
        Ok(rv)
    }

    /// Serialize the geometry as WKT.
    pub fn wkt(&self) -> Result<String> {
        let mut c_wkt = null_mut();
        let rv = unsafe { gdal_sys::OGR_G_ExportToWkt(self.c_geometry(), &mut c_wkt) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_ExportToWkt",
            });
        }
        let wkt = _string(c_wkt);
        unsafe { gdal_sys::OGRFree(c_wkt as *mut c_void) };
        Ok(wkt)
    }

    /// Serializes the geometry to
    /// [WKB](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry#Well-known_binary)
    /// (Well-Known Binary) format.
    pub fn wkb(&self) -> Result<Vec<u8>> {
        let wkb_size = unsafe { gdal_sys::OGR_G_WkbSize(self.c_geometry()) as usize };
        // We default to little-endian for now. A WKB string explicitly indicates the byte
        // order, so this is not a problem for interoperability.
        let byte_order = gdal_sys::OGRwkbByteOrder::wkbNDR;
        let mut wkb = vec![0; wkb_size];
        let rv =
            unsafe { gdal_sys::OGR_G_ExportToWkb(self.c_geometry(), byte_order, wkb.as_mut_ptr()) };
        if rv != gdal_sys::OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_ExportToWkb",
            });
        }
        Ok(wkb)
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

    /// Get point coordinates from a line string or a point geometry.
    ///
    /// `index` is the line string vertex index, from 0 to `point_count()-1`, or `0` when a point.
    ///
    /// Refer: [`OGR_G_GetPoint`](https://gdal.org/api/vector_c_api.html#_CPPv414OGR_G_GetPoint12OGRGeometryHiPdPdPd)
    pub fn get_point(&self, index: i32) -> (f64, f64, f64) {
        let mut x: c_double = 0.;
        let mut y: c_double = 0.;
        let mut z: c_double = 0.;
        unsafe { gdal_sys::OGR_G_GetPoint(self.c_geometry(), index, &mut x, &mut y, &mut z) };
        (x, y, z)
    }

    pub fn get_point_vec(&self) -> Vec<(f64, f64, f64)> {
        let length = unsafe { gdal_sys::OGR_G_GetPointCount(self.c_geometry()) };
        (0..length).map(|i| self.get_point(i)).collect()
    }

    /// Compute the convex hull of this geometry.
    pub fn convex_hull(&self) -> Result<Geometry> {
        let c_geom = unsafe { gdal_sys::OGR_G_ConvexHull(self.c_geometry()) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_ConvexHull"));
        };
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    #[cfg(any(all(major_is_2, minor_ge_1), major_ge_3))]
    pub fn delaunay_triangulation(&self, tolerance: Option<f64>) -> Result<Self> {
        let c_geom = unsafe {
            gdal_sys::OGR_G_DelaunayTriangulation(self.c_geometry(), tolerance.unwrap_or(0.0), 0)
        };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_DelaunayTriangulation"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    pub fn simplify(&self, tolerance: f64) -> Result<Self> {
        let c_geom = unsafe { gdal_sys::OGR_G_Simplify(self.c_geometry(), tolerance) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_Simplify"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Compute buffer of geometry
    ///
    /// The `distance` parameter the buffer distance to be applied. Should be expressed into
    /// the same unit as the coordinates of the geometry. `n_quad_segs` specifies the number
    /// of segments used to approximate a 90 degree (quadrant) of curvature.
    pub fn buffer(&self, distance: f64, n_quad_segs: u32) -> Result<Self> {
        let c_geom =
            unsafe { gdal_sys::OGR_G_Buffer(self.c_geometry(), distance, n_quad_segs as i32) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_Buffer"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    pub fn simplify_preserve_topology(&self, tolerance: f64) -> Result<Self> {
        let c_geom =
            unsafe { gdal_sys::OGR_G_SimplifyPreserveTopology(self.c_geometry(), tolerance) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_SimplifyPreserveTopology"));
        };

        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Get the geometry type ordinal
    ///
    /// Refer: [OGR_G_GetGeometryType](https://gdal.org/api/vector_c_api.html#_CPPv421OGR_G_GetGeometryType12OGRGeometryH)
    pub fn geometry_type(&self) -> OGRwkbGeometryType::Type {
        unsafe { gdal_sys::OGR_G_GetGeometryType(self.c_geometry()) }
    }

    /// Get the WKT name for the type of this geometry.
    ///
    /// Refer: [`OGR_G_GetGeometryName`](https://gdal.org/api/vector_c_api.html#_CPPv421OGR_G_GetGeometryName12OGRGeometryH)
    pub fn geometry_name(&self) -> String {
        // Note: C API makes no statements about this possibly returning null.
        // So we don't have to result wrap this,
        let c_str = unsafe { gdal_sys::OGR_G_GetGeometryName(self.c_geometry()) };
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
    /// Refer: [`OGR_G_GetGeometryCount`](https://gdal.org/api/vector_c_api.html#_CPPv422OGR_G_GetGeometryCount12OGRGeometryH)
    pub fn geometry_count(&self) -> usize {
        let cnt = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) };
        cnt as usize
    }

    /// Get the number of points from a Point or a LineString/LinearRing geometry.
    ///
    /// Only `wkbPoint` or `wkbLineString` may return a non-zero value. Other geometry types will return 0.
    ///
    /// Refer: [`OGR_G_GetPointCount`](https://gdal.org/api/vector_c_api.html#_CPPv419OGR_G_GetPointCount12OGRGeometryH)
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
    pub fn get_geometry(&self, index: usize) -> GeometryRef {
        let geom = unsafe { self.get_unowned_geometry(index) };
        GeometryRef {
            geom,
            _lifetime: PhantomData::default(),
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

    // Transform the geometry inplace (when we own the Geometry)
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

    // Return a new transformed geometry (when the Geometry is owned by a Feature)
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

    /// Transforms a geometrys coordinates into another SpatialRef
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

    pub fn area(&self) -> f64 {
        unsafe { gdal_sys::OGR_G_Area(self.c_geometry()) }
    }

    /// Get the spatial reference system for this geometry.
    ///
    /// Returns `Some(SpatialRef)`, or `None` if one isn't defined.
    ///
    /// Refer [OGR_G_GetSpatialReference](https://gdal.org/doxygen/ogr__api_8h.html#abc393e40282eec3801fb4a4abc9e25bf)
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

    /// Create a copy of self as a `geo-types` geometry.
    pub fn to_geo(&self) -> Result<geo_types::Geometry<f64>> {
        self.try_into()
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
    /// Refer: [OGR_G_MakeValidEx](https://gdal.org/api/vector_c_api.html#_CPPv417OGR_G_MakeValidEx12OGRGeometryH12CSLConstList)
    ///
    /// # Example
    /// ```rust, no_run
    /// use gdal::vector::Geometry;
    /// # fn main() -> gdal::errors::Result<()> {
    /// let src = Geometry::from_wkt("POLYGON ((0 0,10 10,0 10,10 0,0 0))")?;
    /// let dst = src.make_valid(())?;
    /// assert_eq!("MULTIPOLYGON (((10 0,0 0,5 5,10 0)),((10 10,5 5,0 10,10 10)))", dst.wkt()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_valid<O: Into<CslStringList>>(&self, opts: O) -> Result<Geometry> {
        let opts = opts.into();

        fn inner(geom: &Geometry, opts: CslStringList) -> Result<Geometry> {
            #[cfg(all(major_ge_3, minor_ge_4))]
            let c_geom = unsafe { gdal_sys::OGR_G_MakeValidEx(geom.c_geometry(), opts.as_ptr()) };

            #[cfg(not(all(major_ge_3, minor_ge_4)))]
            let c_geom = {
                if !opts.is_empty() {
                    return Err(GdalError::BadArgument(
                        "Options to make_valid require GDAL >= 3.4".into(),
                    ));
                }
                unsafe { gdal_sys::OGR_G_MakeValid(geom.c_geometry()) }
            };

            if c_geom.is_null() {
                Err(_last_null_pointer_err("OGR_G_MakeValid"))
            } else {
                Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
            }
        }
        inner(self, opts)
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
        match self.wkt() {
            Ok(wkt) => f.write_str(wkt.as_str()),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl PartialEq for Geometry {
    fn eq(&self, other: &Self) -> bool {
        unsafe { gdal_sys::OGR_G_Equal(self.c_geometry(), other.c_geometry()) != 0 }
    }
}

impl Eq for Geometry {}

pub fn geometry_type_to_name(ty: OGRwkbGeometryType::Type) -> String {
    let rv = unsafe { gdal_sys::OGRGeometryTypeToName(ty) };
    // If the type is invalid, OGRGeometryTypeToName returns a valid string anyway.
    assert!(!rv.is_null());
    _string(rv)
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

impl Debug for GeometryRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.geom, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_ref::SpatialRef;
    use crate::test_utils::SuppressGDALErrorLog;

    #[test]
    #[allow(clippy::float_cmp)]
    pub fn test_area() {
        let _nolog = SuppressGDALErrorLog::new();
        let geom = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbMultiPolygon).unwrap();
        assert_eq!(geom.area(), 0.0);

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert_eq!(geom.area(), 0.0);

        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let geom = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geom.area().floor(), 25.0);
    }

    #[test]
    pub fn test_is_empty() {
        let geom = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbMultiPolygon).unwrap();
        assert!(geom.is_empty());

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert!(!geom.is_empty());

        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let geom = Geometry::from_wkt(wkt).unwrap();
        assert!(!geom.is_empty());
    }

    #[test]
    pub fn test_create_multipoint_2d() {
        let mut geom = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbMultiPoint).unwrap();
        let mut point = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbPoint).unwrap();
        point.add_point_2d((1.0, 2.0));
        geom.add_geometry(point).unwrap();
        let mut point = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbPoint).unwrap();
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
        let mut geom = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbMultiPoint).unwrap();
        let mut point = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbPoint).unwrap();
        point.add_point((1.0, 2.0, 3.0));
        geom.add_geometry(point).unwrap();
        let mut point = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbPoint).unwrap();
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
        let geom = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbMultiPolygon).unwrap();
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
    pub fn test_wkb() {
        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let orig_geom = Geometry::from_wkt(wkt).unwrap();
        let wkb = orig_geom.wkb().unwrap();
        let new_geom = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(new_geom, orig_geom);
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
    pub fn test_geometry_type_to_name() {
        assert_eq!(
            geometry_type_to_name(::gdal_sys::OGRwkbGeometryType::wkbLineString),
            "Line String"
        );
        // We don't care what it returns when passed an invalid value, just that it doesn't crash.
        geometry_type_to_name(4372521);
    }

    #[test]
    /// Simple clone case.
    pub fn test_make_valid_clone() {
        let src = Geometry::from_wkt("POINT (0 0)").unwrap();
        let dst = src.make_valid(());
        assert!(dst.is_ok());
    }

    #[test]
    /// Un-repairable geometry case
    pub fn test_make_valid_invalid() {
        let _nolog = SuppressGDALErrorLog::new();
        let src = Geometry::from_wkt("LINESTRING (0 0)").unwrap();
        let dst = src.make_valid(());
        assert!(dst.is_err());
    }

    #[test]
    /// Repairable case (self-intersecting)
    pub fn test_make_valid_repairable() {
        let src = Geometry::from_wkt("POLYGON ((0 0,10 10,0 10,10 0,0 0))").unwrap();
        let dst = src.make_valid(());
        assert!(dst.is_ok());
    }

    #[cfg(all(major_ge_3, minor_ge_4))]
    #[test]
    /// Repairable case, but use extended options
    pub fn test_make_valid_ex() {
        let src =
            Geometry::from_wkt("POLYGON ((0 0,0 10,10 10,10 0,0 0),(5 5,15 10,15 0,5 5))").unwrap();
        let dst = src.make_valid(&[("STRUCTURE", "LINEWORK")]);
        assert!(dst.is_ok(), "{dst:?}");
    }
}
