use std::str::raw;
use std::libc::c_char;

pub struct Proj {
    c_proj: *(),
}


#[link(name="proj")]
extern {
    fn pj_init_plus(definition: *c_char) -> *();
    fn pj_free(pj: *());
    fn pj_get_def(pj: *()) -> *c_char;
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
