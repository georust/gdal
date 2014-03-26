use std::libc;
use std::str::raw;


#[link(name = "gdal")]
extern {
    fn GDALVersionInfo(key: *libc::c_char) -> *libc::c_char;
}


pub fn version_info(key: &str) -> ~str {
    let info = key.with_c_str(|c_key| {
        return unsafe { raw::from_c_str(GDALVersionInfo(c_key)) };
    });
    return info;
}


#[test]
fn test_version_info() {
    let release_date = version_info("RELEASE_DATE");
    let release_name = version_info("RELEASE_NAME");
    let version_text = version_info("--version");

    let expected_text: ~str = "GDAL " + release_name + ", " +
        "released " + release_date.slice(0, 4) + "/" +
        release_date.slice(4, 6) + "/" + release_date.slice(6, 8);

    assert_eq!(version_text.into_owned(), expected_text);
}
