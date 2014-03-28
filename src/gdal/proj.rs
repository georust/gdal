use std::str::raw;
use std::libc::{c_int, c_char, c_long, c_double};

pub struct Proj {
    c_proj: *(),
}

//static DEG_TO_RAD: f64 = 0.017453292519943295769236907684886;


#[link(name="proj")]
extern {
    fn pj_init_plus(definition: *c_char) -> *();
    fn pj_free(pj: *());
    fn pj_get_def(pj: *()) -> *c_char;
    fn pj_transform(
        srcdefn: *(),
        dstdefn: *(),
        point_count: c_long,
        point_offset: c_int,
        x: *mut c_double,
        y: *mut c_double,
        z: *mut c_double
    ) -> c_int;
    fn pj_strerrno(code: c_int) -> *c_char;
}


fn error_message(code: c_int) -> ~str {
    unsafe {
        let rv = pj_strerrno(code);
        return raw::from_c_str(rv);
    }
}


impl Proj {
    pub fn new(definition: &str) -> Option<Proj> {
        let c_proj = definition.with_c_str(|c_definition| {
            unsafe { return pj_init_plus(c_definition) }
        });
        return match c_proj.is_null() {
            true  => None,
            false => Some(Proj{c_proj: c_proj}),
        };
    }

    pub fn get_def(&self) -> ~str {
        unsafe {
            let rv = pj_get_def(self.c_proj);
            return raw::from_c_str(rv);
        }
    }

    pub fn project(&self, target: &Proj, x: f64, y: f64) -> (f64, f64) {
        let mut c_x: c_double = x;
        let mut c_y: c_double = y;
        let mut c_z: c_double = 0.;
        unsafe {
            let rv = pj_transform(
                self.c_proj,
                target.c_proj,
                1,
                1,
                &mut c_x,
                &mut c_y,
                &mut c_z
            );
            //if rv != 0 {
            //    println!("{}", error_message(rv));
            //}
            assert!(rv == 0);
        }
        return (c_x, c_y);
    }
}


impl Drop for Proj {
    fn drop(&mut self) {
        unsafe { pj_free(self.c_proj); }
    }
}


#[test]
fn test_new_projection() {
    let wgs84 = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
    let proj = Proj::new(wgs84).unwrap();
    assert_eq!(
        proj.get_def(),
        ~" +proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs +towgs84=0,0,0");
}


fn assert_almost_eq(a: f64, b: f64) {
    let f: f64 = a / b;
    assert!(f < 1.00001);
    assert!(f > 0.99999);
}


#[test]
fn test_transform() {
    let wgs84 = Proj::new("+proj=longlat +datum=WGS84 +no_defs").unwrap();
    let stereo70 = Proj::new(
        "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 " +
        "+x_0=500000 +y_0=500000 +ellps=krass +units=m +no_defs"
        ).unwrap();

    let (lng, lat) = stereo70.project(&wgs84, 500000., 500000.);
    assert_almost_eq(lng, 0.436332);
    assert_almost_eq(lat, 0.802851);

    let (x, y) = wgs84.project(&stereo70, 0.436332, 0.802851);
    assert_almost_eq(x, 500000.);
    assert_almost_eq(y, 500000.);
}
