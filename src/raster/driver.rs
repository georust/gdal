use libc::c_int;
use std::ffi::CString;
use std::sync::{StaticMutex, MUTEX_INIT};
use super::super::geom::Point;
use utils::_string;
use raster::{gdal, RasterDataset};


static mut LOCK: StaticMutex = MUTEX_INIT;
static mut registered_drivers: bool = false;

pub fn _register_drivers() {
    unsafe {
        let _g = LOCK.lock();
        if ! registered_drivers {
            gdal::GDALAllRegister();
            registered_drivers = true;
        }
    }
}


#[allow(missing_copy_implementations)]
pub struct Driver {
    c_driver: *const (),
}


impl Driver {
    pub unsafe fn _with_c_ptr(c_driver: *const ()) -> Driver {
        return Driver{c_driver: c_driver};
    }

    pub unsafe fn _c_ptr(&self) -> *const () {
        return self.c_driver;
    }

    pub fn short_name(&self) -> String {
        let rv = unsafe { gdal::GDALGetDriverShortName(self.c_driver) };
        return _string(rv);
    }

    pub fn long_name(&self) -> String {
        let rv = unsafe { gdal::GDALGetDriverLongName(self.c_driver) };
        return _string(rv);
    }

    pub fn create(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize
    ) -> Option<RasterDataset> {
        use std::ptr::null;
        let c_filename = CString::from_slice(filename.as_bytes());
        let c_dataset = unsafe { gdal::GDALCreate(
                self.c_driver,
                c_filename.as_ptr(),
                size_x as c_int,
                size_y as c_int,
                bands as c_int,
                gdal::GDT_BYTE,
                null()
            ) };
        return match c_dataset.is_null() {
            true  => None,
            false => unsafe { Some(RasterDataset::_with_c_ptr(c_dataset)) },
        };
    }
}


pub fn driver(name: &str) -> Option<Driver> {
    _register_drivers();
    let c_name = CString::from_slice(name.as_bytes());
    let c_driver = unsafe { gdal::GDALGetDriverByName(c_name.as_ptr()) };
    return match c_driver.is_null() {
        true  => None,
        false => Some(Driver{c_driver: c_driver}),
    };
}
