use std::str::raw;
use std::os::getenv;
use std::path::Path;
use std::libc::{c_int, c_char};
use sync::mutex::{StaticMutex, MUTEX_INIT};

static mut LOCK: StaticMutex = MUTEX_INIT;

static mut registered_drivers: bool = false;


struct Dataset {
    c_dataset: *(),
}


#[link(name = "gdal")]
extern {
    fn GDALVersionInfo(key: *c_char) -> *c_char;
    fn GDALOpen(pszFilename: *c_char, eAccess: c_int) -> *();
    fn GDALClose(hDS: *());
    fn GDALGetRasterXSize(hDataset: *()) -> c_int;
    fn GDALGetRasterYSize(hDataset: *()) -> c_int;
    fn GDALGetRasterCount(hDataset: *()) -> c_int;
    fn GDALGetProjectionRef(hDS: *()) -> *c_char;
    fn GDALSetProjection(hDS: *(), pszProjection: *c_char) -> c_int;
    fn GDALAllRegister();
}
static GA_ReadOnly: c_int = 0;
static GA_Update: c_int = 1;


pub fn version_info(key: &str) -> ~str {
    let info = key.with_c_str(|c_key| {
        unsafe {
            let _g = LOCK.lock();
            return raw::from_c_str(GDALVersionInfo(c_key));
        };
    });
    return info;
}


pub fn open(path: &Path) -> Option<Dataset> {
    unsafe {
        let _g = LOCK.lock();
        if ! registered_drivers {
            GDALAllRegister();
            registered_drivers = true;
        }
    }
    let filename = path.as_str().unwrap();
    let c_dataset = filename.with_c_str(|c_filename| {
        unsafe {
            let _g = LOCK.lock();
            return GDALOpen(c_filename, GA_ReadOnly);
        };
    });
    return match c_dataset.is_null() {
        true  => None,
        false => Some(Dataset{c_dataset: c_dataset}),
    };
}


impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe {
            let _g = LOCK.lock();
            GDALClose(self.c_dataset);
        }
    }
}


impl Dataset {
    pub fn get_raster_size(&self) -> (int, int) {
        unsafe {
            let _g = LOCK.lock();
            let size_x = GDALGetRasterXSize(self.c_dataset) as int;
            let size_y = GDALGetRasterYSize(self.c_dataset) as int;
            return (size_x, size_y);
        }
    }

    pub fn get_raster_count(&self) -> int {
        unsafe {
            let _g = LOCK.lock();
            return GDALGetRasterCount(self.c_dataset) as int;
        }
    }

    pub fn get_projection(&self) -> ~str {
        unsafe {
            let _g = LOCK.lock();
            return raw::from_c_str(GDALGetProjectionRef(self.c_dataset));
        }
    }

    pub fn set_projection(&self, projection: &str) {
        projection.with_c_str(|c_projection| {
            unsafe {
                let _g = LOCK.lock();
                GDALSetProjection(self.c_dataset, c_projection);
            }
        });
    }
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


#[test]
fn test_get_raster_size() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let (size_x, size_y) = dataset.get_raster_size();
    assert_eq!(size_x, 100);
    assert_eq!(size_y, 50);
}


#[test]
fn test_get_raster_count() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let count = dataset.get_raster_count();
    assert_eq!(count, 3);
}


#[test]
fn test_get_projection() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    //dataset.set_projection("WGS84");
    let projection = dataset.get_projection();
    assert_eq!(projection.slice(0, 16), "GEOGCS[\"WGS 84\",");
}
