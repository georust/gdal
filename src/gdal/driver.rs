use std::str::raw;
use libc::{c_int, c_char};
use super::register_drivers;
use super::dataset::Dataset;


#[link(name="gdal")]
extern {
    fn GDALCreate(
            hDriver: *(),
            pszFilename: *c_char,
            nXSize: c_int,
            nYSize: c_int,
            nBands: c_int,
            eBandType: c_int,
            papszOptions: **c_char
        ) -> *();
    fn GDALGetDriverByName(pszName: *c_char) -> *();
    fn GDALGetDriverShortName(hDriver: *()) -> *c_char;
    fn GDALGetDriverLongName(hDriver: *()) -> *c_char;
}

static GDT_Byte: c_int = 1;


pub struct Driver {
    c_driver: *(),
}


impl Driver {
    pub unsafe fn with_ptr(c_driver: *()) -> Driver {
        return Driver{c_driver: c_driver};
    }

    pub unsafe fn get_ptr(&self) -> *() {
        return self.c_driver;
    }

    pub fn get_short_name(&self) -> String {
        unsafe {
            let rv = GDALGetDriverShortName(self.c_driver);
            return raw::from_c_str(rv);
        }
    }

    pub fn get_long_name(&self) -> String {
        unsafe {
            let rv = GDALGetDriverLongName(self.c_driver);
            return raw::from_c_str(rv);
        }
    }

    pub fn create(
        &self,
        filename: &str,
        size_x: int,
        size_y: int,
        bands: int
    ) -> Option<Dataset> {
        use std::ptr::null;
        let c_dataset = filename.with_c_str(|c_filename| {
            unsafe {
                return GDALCreate(
                    self.c_driver,
                    c_filename,
                    size_x as c_int,
                    size_y as c_int,
                    bands as c_int,
                    GDT_Byte,
                    null()
                );
            }
        });
        return match c_dataset.is_null() {
            true  => None,
            false => unsafe { Some(Dataset::with_ptr(c_dataset)) },
        };
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
fn test_get_driver_by_name() {
    let missing_driver = get_driver("wtf");
    assert!(missing_driver.is_none());

    let ok_driver = get_driver("GTiff");
    assert!(ok_driver.is_some());
    let driver = ok_driver.unwrap();
    assert_eq!(driver.get_short_name().as_slice(), "GTiff");
    assert_eq!(driver.get_long_name().as_slice(), "GeoTIFF");
}
