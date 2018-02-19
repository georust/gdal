use std::ptr::null_mut;
use libc::{c_int, c_double, c_void};
use std::ffi::CString;
use std::cell::RefCell;
use utils::{_string, _last_null_pointer_err};
use gdal_sys::{self, OGRErr, OGRGeometryH, OGRwkbGeometryType};
use spatial_ref::{SpatialRef, CoordTransform};

use errors::*;

/// OGR Geometry
pub struct Geometry {
    c_geometry_ref: RefCell<Option<OGRGeometryH>>,
    owned: bool,
}

#[derive(Clone,PartialEq,Debug)]
pub enum WkbType {
    WkbUnknown = OGRwkbGeometryType::wkbUnknown as isize,
    WkbPoint = OGRwkbGeometryType::wkbPoint as isize,
    WkbLinestring = OGRwkbGeometryType::wkbLineString as isize,
    WkbPolygon = OGRwkbGeometryType::wkbPolygon as isize,
    WkbMultipoint = OGRwkbGeometryType::wkbMultiPoint as isize,
    WkbMultilinestring = OGRwkbGeometryType::wkbMultiLineString as isize,
    WkbMultipolygon = OGRwkbGeometryType::wkbMultiPolygon as isize,
    WkbGeometrycollection = OGRwkbGeometryType::wkbGeometryCollection as isize,
    WkbLinearring = OGRwkbGeometryType::wkbLinearRing as isize,
}

impl WkbType {
    pub fn from_ogr_type(ogr_type: OGRwkbGeometryType::Type) -> WkbType {
        match ogr_type {
            OGRwkbGeometryType::wkbPoint => WkbType::WkbPoint,
            OGRwkbGeometryType::wkbLineString => WkbType::WkbLinestring,
            OGRwkbGeometryType::wkbPolygon => WkbType::WkbPolygon,
            OGRwkbGeometryType::wkbMultiPoint => WkbType::WkbMultipoint,
            OGRwkbGeometryType::wkbMultiLineString => WkbType::WkbMultilinestring,
            OGRwkbGeometryType::wkbMultiPolygon => WkbType::WkbMultipolygon,
            OGRwkbGeometryType::wkbGeometryCollection => WkbType::WkbGeometrycollection,
            OGRwkbGeometryType::wkbLinearRing => WkbType::WkbLinearring,
            _ => WkbType::WkbUnknown
        }
    }
}

impl Geometry {
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
        Geometry{c_geometry_ref: RefCell::new(None), owned: false}
    }

    pub fn has_gdal_ptr(&self) -> bool {
        self.c_geometry_ref.borrow().is_some()
    }

    pub unsafe fn set_c_geometry(&self, c_geometry: OGRGeometryH) {
        assert!(! self.has_gdal_ptr());
        assert_eq!(self.owned, false);
        *(self.c_geometry_ref.borrow_mut()) = Some(c_geometry);
    }

    unsafe fn with_c_geometry(c_geom: OGRGeometryH, owned: bool) -> Geometry {
        Geometry{
            c_geometry_ref: RefCell::new(Some(c_geom)),
            owned: owned,
        }
    }

    pub fn empty(wkb_type: OGRwkbGeometryType::Type) -> Result<Geometry> {
        let c_geom = unsafe { gdal_sys::OGR_G_CreateGeometry(wkb_type) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_CreateGeometry").into());
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
        let mut c_wkt_ptr = c_wkt.into_raw();
        let mut c_geom = null_mut();
        let rv = unsafe { gdal_sys::OGR_G_CreateFromWkt(&mut c_wkt_ptr, null_mut(), &mut c_geom) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_CreateFromWkt").into());
        }
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Create a rectangular geometry from West, South, East and North values.
    pub fn bbox(w: f64, s: f64, e: f64, n: f64) -> Result<Geometry> {
        Geometry::from_wkt(&format!(
            "POLYGON (({} {}, {} {}, {} {}, {} {}, {} {}))",
            w, n,
            e, n,
            e, s,
            w, s,
            w, n,
        ))
    }

    /// Serialize the geometry as JSON.
    pub fn json(&self) -> Result<String> {
        let c_json = unsafe { gdal_sys::OGR_G_ExportToJson(self.c_geometry()) };
        if c_json.is_null() {
            return Err(_last_null_pointer_err("OGR_G_ExportToJson").into());
        };
        let rv = _string(c_json);
        unsafe { gdal_sys::VSIFree(c_json as *mut c_void) };
        Ok(rv)
    }

    /// Serialize the geometry as WKT.
    pub fn wkt(&self) -> Result<String> {
        let mut c_wkt = null_mut();
        let _err = unsafe { gdal_sys::OGR_G_ExportToWkt(self.c_geometry(), &mut c_wkt) };
        if _err != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(_err, "OGR_G_ExportToWkt").into());
        }
        let wkt = _string(c_wkt);
        unsafe { gdal_sys::OGRFree(c_wkt as *mut c_void) };
        Ok(wkt)
    }

    pub unsafe fn c_geometry(&self) -> OGRGeometryH {
        self.c_geometry_ref.borrow().unwrap()
    }

    pub unsafe fn into_c_geometry(mut self) -> OGRGeometryH {
        assert!(self.owned);
        self.owned = false;
        self.c_geometry()
    }

    pub fn set_point_2d(&mut self, i: usize, p: (f64, f64)) {
        let (x, y) = p;
        unsafe { gdal_sys::OGR_G_SetPoint_2D(
            self.c_geometry(),
            i as c_int,
            x as c_double,
            y as c_double,
        ) };
    }

    pub fn get_point(&self, i: i32) -> (f64, f64, f64) {
        let mut x: c_double = 0.;
        let mut y: c_double = 0.;
        let mut z: c_double = 0.;
        unsafe { gdal_sys::OGR_G_GetPoint(self.c_geometry(), i, &mut x, &mut y, &mut z) };
        (x as f64, y as f64, z as f64)
    }

    pub fn get_point_vec(&self) -> Vec<(f64, f64, f64)> {
        let length = unsafe{ gdal_sys::OGR_G_GetPointCount(self.c_geometry()) };
        (0..length).map(|i| self.get_point(i)).collect()
    }

    /// Compute the convex hull of this geometry.
    pub fn convex_hull(&self) -> Result<Geometry> {
        let c_geom = unsafe { gdal_sys::OGR_G_ConvexHull(self.c_geometry()) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_ConvexHull").into());
        };
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    pub fn geometry_type(&self) -> WkbType {
        let ogr_type = unsafe { gdal_sys::OGR_G_GetGeometryType(self.c_geometry()) };
        WkbType::from_ogr_type(ogr_type)
    }

    pub fn geometry_count(&self) -> usize {
        let cnt = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) };
        cnt as usize
    }

    pub unsafe fn _get_geometry(&self, n: usize) -> Geometry {
        // get the n-th sub-geometry as a non-owned Geometry; don't keep this
        // object for long.
        let c_geom = gdal_sys::OGR_G_GetGeometryRef(self.c_geometry(), n as c_int);
        Geometry::with_c_geometry(c_geom, false)
    }

    pub fn add_geometry(&mut self, mut sub: Geometry) -> Result<()> {
        assert!(sub.owned);
        sub.owned = false;
        let rv = unsafe { gdal_sys::OGR_G_AddGeometryDirectly(
            self.c_geometry(),
            sub.c_geometry(),
        ) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_AddGeometryDirectly").into());
        }
        Ok(())
    }

    // Transform the geometry inplace (when we own the Geometry)
    pub fn transform_inplace(&self, htransform: &CoordTransform) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_G_Transform(
            self.c_geometry(),
            htransform.to_c_hct()
        ) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_Transform").into());
        }
        Ok(())
    }

    // Return a new transformed geometry (when the Geometry is owned by a Feature)
    pub fn transform(&self, htransform: &CoordTransform) -> Result<Geometry> {
        let new_c_geom = unsafe { gdal_sys::OGR_G_Clone(self.c_geometry()) };
        let rv = unsafe { gdal_sys::OGR_G_Transform(new_c_geom, htransform.to_c_hct()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_Transform").into());
        }
        Ok(unsafe { Geometry::with_c_geometry(new_c_geom, true) } )
    }

    pub fn transform_to_inplace(&self, spatial_ref: &SpatialRef) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_G_TransformTo(
            self.c_geometry(),
            spatial_ref.to_c_hsrs()
        ) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_TransformTo").into());
        }
        Ok(())
    }

    pub fn transform_to(&self, spatial_ref: &SpatialRef) -> Result<Geometry> {
        let new_c_geom = unsafe { gdal_sys::OGR_G_Clone(self.c_geometry()) };
        let rv = unsafe { gdal_sys::OGR_G_TransformTo(new_c_geom, spatial_ref.to_c_hsrs()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_TransformTo").into());
        }
        Ok(unsafe { Geometry::with_c_geometry(new_c_geom, true) } )
    }

    pub fn area(&self) -> f64 {
        unsafe { gdal_sys::OGR_G_Area(self.c_geometry()) }
    }

    /// May or may not contain a reference to a SpatialRef: if not, it returns
    /// an `Ok(None)`; if it does, it tries to build a SpatialRef. If that
    /// succeeds, it returns an Ok(Some(SpatialRef)), otherwise, you get the
    /// Err.
    ///
    pub fn spatial_reference(&self) -> Option<SpatialRef> {
        let c_spatial_ref = unsafe { gdal_sys::OGR_G_GetSpatialReference(self.c_geometry()) };

        if c_spatial_ref.is_null() {
            None
        } else {
            match SpatialRef::from_c_obj(c_spatial_ref) {
                Ok(sr) => Some(sr),
                Err(_) => None
            }
        }
    }

    pub fn set_spatial_reference(&mut self, spatial_ref: SpatialRef) {
        unsafe { gdal_sys::OGR_G_AssignSpatialReference(self.c_geometry(), spatial_ref.to_c_hsrs()) };
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
        let new_c_geom = unsafe { gdal_sys::OGR_G_Clone(c_geometry.unwrap())};
        unsafe { Geometry::with_c_geometry(new_c_geom, true) }
    }
}

#[cfg(test)]
mod tests {
    use super::Geometry;
    use spatial_ref::SpatialRef;

    #[test]
    pub fn test_area() {
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
    pub fn test_spatial_reference() {
        let geom = Geometry::empty(::gdal_sys::OGRwkbGeometryType::wkbMultiPolygon).unwrap();
        assert!(geom.spatial_reference().is_none());

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert!(geom.spatial_reference().is_none());

        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let mut geom = Geometry::from_wkt(wkt).unwrap();
        assert!(geom.spatial_reference().is_none());

        let srs = SpatialRef::from_epsg(4326).unwrap();
        geom.set_spatial_reference(srs);
        assert!(geom.spatial_reference().is_some());
    }
}
