use std::str::raw;
use std::os::getenv;
use std::path::Path;
use std::libc::{c_int, c_char};
use sync::mutex::{StaticMutex, MUTEX_INIT};


#[link(name = "gdal")]
extern {
    fn GDALVersionInfo(key: *c_char) -> *c_char;
    fn GDALOpen(pszFilename: *c_char, eAccess: c_int) -> *();
    fn GDALClose(hDS: *());
    fn GDALGetDatasetDriver(hDataset: *()) -> *();
    fn GDALGetRasterXSize(hDataset: *()) -> c_int;
    fn GDALGetRasterYSize(hDataset: *()) -> c_int;
    fn GDALGetRasterCount(hDataset: *()) -> c_int;
    fn GDALGetProjectionRef(hDS: *()) -> *c_char;
    fn GDALSetProjection(hDS: *(), pszProjection: *c_char) -> c_int;
    fn GDALAllRegister();
    fn GDALGetDriverByName(pszName: *c_char) -> *();
    fn GDALGetDriverShortName(hDriver: *()) -> *c_char;
    fn GDALGetDriverLongName(hDriver: *()) -> *c_char;
}

static GA_ReadOnly: c_int = 0;
static GA_Update: c_int = 1;


static mut LOCK: StaticMutex = MUTEX_INIT;
static mut registered_drivers: bool = false;

fn register_drivers() {
    unsafe {
        let _g = LOCK.lock();
        if ! registered_drivers {
            GDALAllRegister();
            registered_drivers = true;
        }
    }
}


pub fn version_info(key: &str) -> ~str {
    let info = key.with_c_str(|c_key| {
        unsafe {
            let rv = GDALVersionInfo(c_key);
            return raw::from_c_str(rv);
        };
    });
    return info;
}


struct Dataset {
    c_dataset: *(),
}


impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe { GDALClose(self.c_dataset); }
    }
}


impl Dataset {
    pub fn get_raster_size(&self) -> (int, int) {
        let size_x = unsafe { GDALGetRasterXSize(self.c_dataset) } as int;
        let size_y = unsafe { GDALGetRasterYSize(self.c_dataset) } as int;
        return (size_x, size_y);
    }

    pub fn get_driver(&self) -> Driver {
        let c_driver = unsafe { GDALGetDatasetDriver(self.c_dataset) };
        return Driver{c_driver: c_driver};
    }

    pub fn get_raster_count(&self) -> int {
        return unsafe { GDALGetRasterCount(self.c_dataset) } as int;
    }

    pub fn get_projection(&self) -> ~str {
        unsafe {
            let rv = GDALGetProjectionRef(self.c_dataset);
            return raw::from_c_str(rv);
        }
    }

    pub fn set_projection(&self, projection: &str) {
        projection.with_c_str(|c_projection| {
            unsafe { GDALSetProjection(self.c_dataset, c_projection) };
        });
    }
}


pub fn open(path: &Path) -> Option<Dataset> {
    register_drivers();
    let filename = path.as_str().unwrap();
    let c_dataset = filename.with_c_str(|c_filename| {
        return unsafe { GDALOpen(c_filename, GA_ReadOnly) };
    });
    return match c_dataset.is_null() {
        true  => None,
        false => Some(Dataset{c_dataset: c_dataset}),
    };
}


struct Driver {
    c_driver: *(),
}


impl Driver {
    pub fn get_short_name(&self) -> ~str {
        unsafe {
            let rv = GDALGetDriverShortName(self.c_driver);
            return raw::from_c_str(rv);
        }
    }

    pub fn get_long_name(&self) -> ~str {
        unsafe {
            let rv = GDALGetDriverLongName(self.c_driver);
            return raw::from_c_str(rv);
        }
    }
}


pub fn get_driver(name: &str) -> Option<Driver> {
    register_drivers();
    let c_driver = name.with_c_str(|c_name| {
        return unsafe { GDALGetDriverByName(c_name) };
    });
    return match c_driver.is_null() {
        true  => None,
        false => Some(Driver{c_driver: c_driver}),
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


#[test]
fn test_get_dataset_driver() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let driver = dataset.get_driver();
    assert_eq!(driver.get_short_name(), ~"JPEG");
    assert_eq!(driver.get_long_name(), ~"JPEG JFIF");
}


#[test]
fn test_get_driver_by_name() {
    let missing_driver = get_driver("wtf");
    assert!(missing_driver.is_none());

    let ok_driver = get_driver("GTiff");
    assert!(ok_driver.is_some());
    let driver = ok_driver.unwrap();
    assert_eq!(driver.get_short_name(), ~"GTiff");
    assert_eq!(driver.get_long_name(), ~"GeoTIFF");
}
