use std::libc;
use std::str::raw;


#[link(name = "gdal")]
extern {
    fn GDALVersionInfo(key: *libc::c_char) -> *libc::c_char;
}


pub fn version_info() -> ~str {
    let key = "--version";
    let info = key.with_c_str(|c_key| {
        return unsafe { raw::from_c_str(GDALVersionInfo(c_key)) };
    });
    return info;
}


#[test]
fn test_version_info() {
    let rv = version_info();
    assert_eq!(rv.slice(0, 4), "GDAL");
}
