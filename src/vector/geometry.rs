use std::ptr::null;
use libc::{c_char, c_int, c_double};
use std::ffi::CString;
use std::cell::RefCell;
use utils::_string;
use vector::ogr;
use geo;

/// OGR Geometry
pub struct Geometry {
    c_geometry_ref: RefCell<Option<*const ()>>,
    owned: bool,
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
        return Geometry{c_geometry_ref: RefCell::new(None), owned: false};
    }

    pub fn has_gdal_ptr(&self) -> bool {
        return self.c_geometry_ref.borrow().is_some();
    }

    pub unsafe fn set_c_geometry(&self, c_geometry: *const ()) {
        assert!(! self.has_gdal_ptr());
        assert_eq!(self.owned, false);
        *(self.c_geometry_ref.borrow_mut()) = Some(c_geometry);
    }

    unsafe fn with_c_geometry(c_geom: *const(), owned: bool) -> Geometry {
        return Geometry{
            c_geometry_ref: RefCell::new(Some(c_geom)),
            owned: owned,
        };
    }

    pub fn empty(wkb_type: c_int) -> Geometry {
        let c_geom = unsafe { ogr::OGR_G_CreateGeometry(wkb_type) };
        assert!(c_geom != null());
        return unsafe { Geometry::with_c_geometry(c_geom, true) };
    }

    /// Create a geometry by parsing a
    /// [WKT](https://en.wikipedia.org/wiki/Well-known_text) string.
    pub fn from_wkt(wkt: &str) -> Geometry {
        let c_wkt = CString::new(wkt.as_bytes()).unwrap();
        let mut c_wkt_ptr: *const c_char = c_wkt.as_ptr();
        let mut c_geom: *const () = null();
        let rv = unsafe { ogr::OGR_G_CreateFromWkt(&mut c_wkt_ptr, null(), &mut c_geom) };
        assert_eq!(rv, ogr::OGRERR_NONE);
        return unsafe { Geometry::with_c_geometry(c_geom, true) };
    }

    /// Create a rectangular geometry from West, South, East and North values.
    pub fn bbox(w: f64, s: f64, e: f64, n: f64) -> Geometry {
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
    pub fn json(&self) -> String {
        let c_json = unsafe { ogr::OGR_G_ExportToJson(self.c_geometry()) };
        let rv = _string(c_json);
        unsafe { ogr::VSIFree(c_json as *mut ()) };
        return rv;
    }

    /// Serialize the geometry as WKT.
    pub fn wkt(&self) -> String {
        let mut c_wkt: *const c_char = null();
        let _err = unsafe { ogr::OGR_G_ExportToWkt(self.c_geometry(), &mut c_wkt) };
        assert_eq!(_err, ogr::OGRERR_NONE);
        let wkt = _string(c_wkt);
        unsafe { ogr::OGRFree(c_wkt as *mut ()) };
        return wkt;
    }

    pub unsafe fn c_geometry(&self) -> *const () {
        return self.c_geometry_ref.borrow().unwrap();
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
        return (0..length).map(|i| self.get_point(i)).collect();
    }

    /// Compute the convex hull of this geometry.
    pub fn convex_hull(&self) -> Geometry {
        let c_geom = unsafe { ogr::OGR_G_ConvexHull(self.c_geometry()) };
        return unsafe { Geometry::with_c_geometry(c_geom, true) };
    }

    unsafe fn _get_geometry(&self, n: usize) -> Geometry {
        // get the n-th sub-geometry as a non-owned Geometry; don't keep this
        // object for long.
        let c_geom = ogr::OGR_G_GetGeometryRef(self.c_geometry(), n as c_int);
        return Geometry::with_c_geometry(c_geom, false);
    }

    pub fn add_geometry(&mut self, mut sub: Geometry) {
        assert!(sub.owned);
        sub.owned = false;
        let rv = unsafe { ogr::OGR_G_AddGeometryDirectly(
            self.c_geometry(),
            sub.c_geometry(),
        ) };
        assert_eq!(rv, ogr::OGRERR_NONE);
    }
}

impl geo::ToGeo for Geometry {
    fn to_geo(&self) -> geo::Geometry {
        let geometry_type = unsafe { ogr::OGR_G_GetGeometryType(self.c_geometry()) };

        let ring = |n: usize| {
            let ring = unsafe { self._get_geometry(n) };
            return match ring.to_geo() {
                geo::Geometry::LineString(r) => r,
                _ => panic!("Expected to get a LineString")
            };
        };

        match geometry_type {
            ogr::WKB_POINT => {
                let (x, y, _) = self.get_point(0);
                geo::Geometry::Point(geo::Point(geo::Coordinate{x: x, y: y}))
            },
            ogr::WKB_MULTIPOINT => {
                let point_count = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let coords = (0..point_count)
                    .map(|n| {
                        match unsafe { self._get_geometry(n) }.to_geo() {
                            geo::Geometry::Point(p) => p,
                            _ => panic!("Expected to get a Point")
                        }
                    })
                    .collect();
                geo::Geometry::MultiPoint(geo::MultiPoint(coords))
            },
            ogr::WKB_LINESTRING => {
                let coords = self.get_point_vec().iter()
                    .map(|&(x, y, _)| geo::Point(geo::Coordinate{x: x, y: y}))
                    .collect();
                geo::Geometry::LineString(geo::LineString(coords))
            },
            ogr::WKB_POLYGON => {
                let ring_count = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let outer = ring(0);
                let holes = (1..ring_count).map(|n| ring(n)).collect();
                geo::Geometry::Polygon(geo::Polygon(outer, holes))
            },
            _ => panic!("Unknown geometry type")
        }
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        if self.owned {
            let c_geometry = self.c_geometry_ref.borrow();
            unsafe { ogr::OGR_G_DestroyGeometry(c_geometry.unwrap() as *mut ()) };
        }
    }
}


pub trait ToGdal {
    fn to_gdal(&self) -> Geometry;
}


impl ToGdal for geo::Point {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_POINT);
        let &geo::Point(coordinate) = self;
        geom.set_point_2d(0, (coordinate.x, coordinate.y));
        return geom;
    }
}

fn geometry_with_points(wkb_type: c_int, points: &geo::LineString) -> Geometry {
    let mut geom = Geometry::empty(wkb_type);
    let &geo::LineString(ref linestring) = points;
    for (i, &geo::Point(coordinate)) in linestring.iter().enumerate() {
        geom.set_point_2d(i, (coordinate.x, coordinate.y));
    }
    return geom;
}

impl ToGdal for geo::LineString {
    fn to_gdal(&self) -> Geometry {
        geometry_with_points(ogr::WKB_LINESTRING, self)
    }
}

impl ToGdal for geo::Polygon {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_POLYGON);
        let &geo::Polygon(ref outer, ref holes) = self;
        geom.add_geometry(geometry_with_points(ogr::WKB_LINEARRING, outer));
        for ring in holes.iter() {
            geom.add_geometry(geometry_with_points(ogr::WKB_LINEARRING, ring));
        }
        return geom;
    }
}

impl ToGdal for geo::Geometry {
    fn to_gdal(&self) -> Geometry {
        return match *self {
            geo::Geometry::Point(ref c) => c.to_gdal(),
            geo::Geometry::LineString(ref c) => c.to_gdal(),
            geo::Geometry::Polygon(ref c) => c.to_gdal(),
            _ => panic!("Unknown geometry type")
        }
    }
}


#[cfg(test)]
mod tests {
    use vector::{Geometry, ToGdal};
    use geo;
    use geo::ToGeo;

    #[test]
    fn test_import_export_point() {
        let wkt = "POINT (1 2)";
        let coord = geo::Coordinate{x: 1., y: 2.};
        let geo = geo::Geometry::Point(geo::Point(coord));

        assert_eq!(Geometry::from_wkt(wkt).to_geo(), geo);
        assert_eq!(geo.to_gdal().wkt(), wkt);
    }

    #[test]
    fn test_import_export_multipoint() {
        let wkt = "MULTIPOINT ((0 0),(0 1),(1 2))";
        let coord = vec!(
            geo::Point(geo::Coordinate{x: 0., y: 0.}),
            geo::Point(geo::Coordinate{x: 0., y: 1.}),
            geo::Point(geo::Coordinate{x: 1., y: 2.}),
        );
        let geo = geo::Geometry::MultiPoint(geo::MultiPoint(coord));

        assert_eq!(Geometry::from_wkt(wkt).to_geo(), geo);
    }

    #[test]
    fn test_import_export_linestring() {
        let wkt = "LINESTRING (0 0,0 1,1 2)";
        let coord = vec!(
            geo::Point(geo::Coordinate{x: 0., y: 0.}),
            geo::Point(geo::Coordinate{x: 0., y: 1.}),
            geo::Point(geo::Coordinate{x: 1., y: 2.}),
        );
        let geo = geo::Geometry::LineString(geo::LineString(coord));

        assert_eq!(Geometry::from_wkt(wkt).to_geo(), geo);
        assert_eq!(geo.to_gdal().wkt(), wkt);
    }

    fn square(x0: isize, y0: isize, x1: isize, y1: isize) -> geo::LineString {
        geo::LineString(vec!(
            geo::Point(geo::Coordinate{x: x0 as f64, y: y0 as f64}),
            geo::Point(geo::Coordinate{x: x0 as f64, y: y1 as f64}),
            geo::Point(geo::Coordinate{x: x1 as f64, y: y1 as f64}),
            geo::Point(geo::Coordinate{x: x1 as f64, y: y0 as f64}),
            geo::Point(geo::Coordinate{x: x0 as f64, y: y0 as f64}),
        ))
    }

    #[test]
    fn test_import_export_polygon() {
        let wkt = "POLYGON ((0 0,0 5,5 5,5 0,0 0),\
                            (1 1,1 2,2 2,2 1,1 1),\
                            (3 3,3 4,4 4,4 3,3 3))";
        let outer = square(0, 0, 5, 5);
        let holes = vec!(square(1, 1, 2, 2), square(3, 3, 4, 4));
        let geo = geo::Geometry::Polygon(geo::Polygon(outer, holes));

        assert_eq!(Geometry::from_wkt(wkt).to_geo(), geo);
        assert_eq!(geo.to_gdal().wkt(), wkt);
    }
}
