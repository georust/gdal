use std::libc;
use std::str::raw;
use std::os::getenv;
use std::path::Path;
use std::libc::c_int;


struct Dataset {
    c_dataset: *(),
}


#[link(name = "gdal")]
extern {
    fn GDALVersionInfo(key: *libc::c_char) -> *libc::c_char;
    fn GDALOpen(pszFilename: *libc::c_char, eAccess: c_int) -> *();
    fn GDALAllRegister();
}
static GA_ReadOnly: c_int = 0;
static GA_Update: c_int = 1;


pub fn version_info(key: &str) -> ~str {
    let info = key.with_c_str(|c_key| {
        return unsafe { raw::from_c_str(GDALVersionInfo(c_key)) };
    });
    return info;
}


pub fn open(path: &Path) -> Option<Dataset> {
    unsafe { GDALAllRegister(); }  // TODO call once
    let filename = path.as_str().unwrap();
    let c_dataset = filename.with_c_str(|c_filename| {
        return unsafe { GDALOpen(c_filename, GA_ReadOnly) };
    });
    return match c_dataset.is_null() {
        true  => None,
        false => Some(Dataset{c_dataset: c_dataset}),
    };
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


fn fixture_path(name: &str) -> Path {
    let envvar = "RUSTILES_TEST_FIXTURES";
    let fixtures = match getenv(envvar) {
        Some(p) => Path::new(p),
        None => fail!("Environment variable {} not set", envvar)
    };
    let rv = fixtures.join(name);
    return rv;
}


#[test]
fn test_open() {
    let dataset = open(&fixture_path("tinymarble.jpeg"));
    assert!(dataset.is_some());

    let missing_dataset = open(&fixture_path("no_such_file.jpeg"));
    assert!(missing_dataset.is_none());
}
