use std::ptr::null;
use libc::c_char;
use std::ffi::CString;
use utils::_string;
use vector::ogr;


pub struct Geometry {
    c_geometry: *const (),
}


impl Geometry {
    pub fn bbox(w: f64, s: f64, e: f64, n: f64) -> Geometry {
        let wkt = format!(
            "POLYGON (({} {}, {} {}, {} {}, {} {}, {} {}))",
            w, n,
            e, n,
            e, s,
            w, s,
            w, n,
        );
        let c_wkt = CString::from_slice(wkt.as_bytes());
        let mut c_wkt_ptr: *const c_char = c_wkt.as_ptr();
        let mut c_geom: *const () = null();
        let rv = unsafe { ogr::OGR_G_CreateFromWkt(&mut c_wkt_ptr, null(), &mut c_geom) };
        assert_eq!(rv, ogr::OGRERR_NONE);
        return Geometry{c_geometry: c_geom};
    }

    pub fn json(&self) -> String {
        let c_json = unsafe { ogr::OGR_G_ExportToJson(self.c_geometry) };
        let rv = _string(c_json);
        unsafe { ogr::VSIFree(c_json as *mut ()) };
        return rv;
    }

    pub unsafe fn c_geometry(&self) -> *const () {
        return self.c_geometry;
    }
}


impl Drop for Geometry {
    fn drop(&mut self) {
        unsafe { ogr::OGR_G_DestroyGeometry(self.c_geometry as *mut ()) };
    }
}
