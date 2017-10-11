use std::ptr::null;
use libc::{c_char, c_int, c_double, c_void};
use std::ffi::CString;
use std::cell::RefCell;
use utils::{_string, _last_null_pointer_err};
use gdal_sys::{ogr, ogr_enums};
use spatial_ref::{SpatialRef, CoordTransform};

use errors::*;

/// OGR Geometry
pub struct Geometry {
    c_geometry_ref: RefCell<Option<*const c_void>>,
    owned: bool,
}

#[derive(Clone,PartialEq,Debug)]
pub enum WkbType {
    WkbUnknown = ogr::WKB_UNKNOWN as isize,
    WkbPoint = ogr::WKB_POINT as isize,
    WkbLinestring = ogr::WKB_LINESTRING as isize,
    WkbPolygon = ogr::WKB_POLYGON as isize,
    WkbMultipoint = ogr::WKB_MULTIPOINT as isize,
    WkbMultilinestring = ogr::WKB_MULTILINESTRING as isize,
    WkbMultipolygon = ogr::WKB_MULTIPOLYGON as isize,
    WkbGeometrycollection = ogr::WKB_GEOMETRYCOLLECTION as isize,
    WkbLinearring = ogr::WKB_LINEARRING as isize,
}

impl WkbType {
    pub fn from_ogr_type(ogr_type: c_int) -> WkbType {
        match ogr_type {
            ogr::WKB_UNKNOWN => WkbType::WkbUnknown,
            ogr::WKB_POINT => WkbType::WkbPoint,
            ogr::WKB_LINESTRING => WkbType::WkbLinestring,
            ogr::WKB_POLYGON => WkbType::WkbPolygon,
            ogr::WKB_MULTIPOINT => WkbType::WkbMultipoint,
            ogr::WKB_MULTILINESTRING => WkbType::WkbMultilinestring,
            ogr::WKB_MULTIPOLYGON => WkbType::WkbMultipolygon,
            ogr::WKB_GEOMETRYCOLLECTION => WkbType::WkbGeometrycollection,
            ogr::WKB_LINEARRING => WkbType::WkbLinearring,
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

    pub unsafe fn set_c_geometry(&self, c_geometry: *const c_void) {
        assert!(! self.has_gdal_ptr());
        assert_eq!(self.owned, false);
        *(self.c_geometry_ref.borrow_mut()) = Some(c_geometry);
    }

    unsafe fn with_c_geometry(c_geom: *const c_void, owned: bool) -> Geometry {
        Geometry{
            c_geometry_ref: RefCell::new(Some(c_geom)),
            owned: owned,
        }
    }

    pub fn empty(wkb_type: c_int) -> Result<Geometry> {
        let c_geom = unsafe { ogr::OGR_G_CreateGeometry(wkb_type) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_CreateGeometry").into());
        };
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Create a geometry by parsing a
    /// [WKT](https://en.wikipedia.org/wiki/Well-known_text) string.
    pub fn from_wkt(wkt: &str) -> Result<Geometry> {
        let c_wkt = CString::new(wkt)?;
        let mut c_wkt_ptr: *const c_char = c_wkt.as_ptr();
        let mut c_geom: *const c_void = null();
        let rv = unsafe { ogr::OGR_G_CreateFromWkt(&mut c_wkt_ptr, null(), &mut c_geom) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
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
        let c_json = unsafe { ogr::OGR_G_ExportToJson(self.c_geometry()) };
        if c_json.is_null() {
            return Err(_last_null_pointer_err("OGR_G_ExportToJson").into());
        };
        let rv = _string(c_json);
        unsafe { ogr::VSIFree(c_json as *mut c_void) };
        Ok(rv)
    }

    /// Serialize the geometry as WKT.
    pub fn wkt(&self) -> Result<String> {
        let mut c_wkt: *const c_char = null();
        let _err = unsafe { ogr::OGR_G_ExportToWkt(self.c_geometry(), &mut c_wkt) };
        if _err != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(_err, "OGR_G_ExportToWkt").into());
        }
        let wkt = _string(c_wkt);
        unsafe { ogr::OGRFree(c_wkt as *mut c_void) };
        Ok(wkt)
    }

    pub unsafe fn c_geometry(&self) -> *const c_void {
        self.c_geometry_ref.borrow().unwrap()
    }

    pub unsafe fn into_c_geometry(mut self) -> *const c_void {
        assert!(self.owned);
        self.owned = false;
        return self.c_geometry();
    }

    pub fn set_point_2d(&mut self, i: usize, p: (f64, f64)) {
        let (x, y) = p;
        unsafe { ogr::OGR_G_SetPoint_2D(
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
        unsafe { ogr::OGR_G_GetPoint(self.c_geometry(), i, &mut x, &mut y, &mut z) };
        return (x as f64, y as f64, z as f64);
    }

    pub fn get_point_vec(&self) -> Vec<(f64, f64, f64)> {
        let length = unsafe{ ogr::OGR_G_GetPointCount(self.c_geometry()) };
        (0..length).map(|i| self.get_point(i)).collect()
    }

    /// Compute the convex hull of this geometry.
    pub fn convex_hull(&self) -> Result<Geometry> {
        let c_geom = unsafe { ogr::OGR_G_ConvexHull(self.c_geometry()) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_ConvexHull").into());
        };
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    pub fn geometry_type(&self) -> WkbType {
        let ogr_type = unsafe { ogr::OGR_G_GetGeometryType(self.c_geometry()) };
        WkbType::from_ogr_type(ogr_type)
    }

    pub fn geometry_count(&self) -> usize {
        let cnt = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) };
        cnt as usize
    }

    pub unsafe fn _get_geometry(&self, n: usize) -> Geometry {
        // get the n-th sub-geometry as a non-owned Geometry; don't keep this
        // object for long.
        let c_geom = ogr::OGR_G_GetGeometryRef(self.c_geometry(), n as c_int);
        return Geometry::with_c_geometry(c_geom, false);
    }

    pub fn add_geometry(&mut self, mut sub: Geometry) -> Result<()> {
        assert!(sub.owned);
        sub.owned = false;
        let rv = unsafe { ogr::OGR_G_AddGeometryDirectly(
            self.c_geometry(),
            sub.c_geometry(),
        ) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_AddGeometryDirectly").into());
        }
        Ok(())
    }

    // Transform the geometry inplace (when we own the Geometry)
    pub fn transform_inplace(&self, htransform: &CoordTransform) -> Result<()> {
        let rv = unsafe { ogr::OGR_G_Transform(
            self.c_geometry(),
            htransform.to_c_hct()
        ) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_Transform").into());
        }
        Ok(())
    }

    // Return a new transformed geometry (when the Geometry is owned by a Feature)
    pub fn transform(&self, htransform: &CoordTransform) -> Result<Geometry> {
        let new_c_geom = unsafe { ogr::OGR_G_Clone(self.c_geometry()) };
        let rv = unsafe { ogr::OGR_G_Transform(new_c_geom, htransform.to_c_hct()) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_Transform").into());
        }
        Ok(unsafe { Geometry::with_c_geometry(new_c_geom, true) } )
    }

    pub fn transform_to_inplace(&self, spatial_ref: &SpatialRef) -> Result<()> {
        let rv = unsafe { ogr::OGR_G_TransformTo(
            self.c_geometry(),
            spatial_ref.to_c_hsrs()
        ) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_TransformTo").into());
        }
        Ok(())
    }

    pub fn transform_to(&self, spatial_ref: &SpatialRef) -> Result<Geometry> {
        let new_c_geom = unsafe { ogr::OGR_G_Clone(self.c_geometry()) };
        let rv = unsafe { ogr::OGR_G_TransformTo(new_c_geom, spatial_ref.to_c_hsrs()) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_TransformTo").into());
        }
        Ok(unsafe { Geometry::with_c_geometry(new_c_geom, true) } )
    }

    pub fn area(&self) -> f64 {
        unsafe { ogr::OGR_G_Area(self.c_geometry()) }
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        if self.owned {
            let c_geometry = self.c_geometry_ref.borrow();
            unsafe { ogr::OGR_G_DestroyGeometry(c_geometry.unwrap() as *mut c_void) };
        }
    }
}

impl Clone for Geometry {
    fn clone(&self) -> Geometry {
        // assert!(self.has_gdal_ptr());
        let c_geometry = self.c_geometry_ref.borrow();
        let new_c_geom = unsafe { ogr::OGR_G_Clone(c_geometry.unwrap())};
        unsafe { Geometry::with_c_geometry(new_c_geom, true) }
    }
}

#[cfg(test)]
mod tests {
    use super::Geometry;

    #[test]
    pub fn test_area() {
        let geom = Geometry::empty(::gdal_sys::ogr::WKB_MULTIPOLYGON).unwrap();
        assert_eq!(geom.area(), 0.0);

        let geom = Geometry::from_wkt("POINT(0 0)").unwrap();
        assert_eq!(geom.area(), 0.0);

        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let geom = Geometry::from_wkt(wkt).unwrap();
        assert_eq!(geom.area().floor(), 25.0);
    }
}
