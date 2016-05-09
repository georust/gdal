use libc::{c_int, c_void};
use std::ffi::CString;
use std::sync::{Once, ONCE_INIT};
use utils::_string;
use raster::{gdal, Dataset};
use raster::types::GdalType;


static START: Once = ONCE_INIT;
static mut registered_drivers: bool = false;

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            gdal::GDALAllRegister();
            registered_drivers = true;
        });
    }
}


#[allow(missing_copy_implementations)]
pub struct Driver {
    c_driver: *const c_void,
}


impl Driver {
    pub fn get(name: &str) -> Option<Driver> {
        _register_drivers();
        let c_name = CString::new(name.as_bytes()).unwrap();
        let c_driver = unsafe { gdal::GDALGetDriverByName(c_name.as_ptr()) };
        return match c_driver.is_null() {
            true  => None,
            false => Some(Driver{c_driver: c_driver}),
        };
    }

    pub unsafe fn _with_c_ptr(c_driver: *const c_void) -> Driver {
        return Driver{c_driver: c_driver};
    }

    pub unsafe fn _c_ptr(&self) -> *const c_void {
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
    ) -> Option<Dataset> {
        self.create_with_band_type::<u8>(
            filename,
            size_x,
            size_y,
            bands,
        )
    }

    pub fn create_with_band_type<T: GdalType>(
        &self,
        filename: &str,
        size_x: isize,
        size_y: isize,
        bands: isize,
    ) -> Option<Dataset> {
        use std::ptr::null;
        let c_filename = CString::new(filename.as_bytes()).unwrap();
        let c_dataset = unsafe { gdal::GDALCreate(
                self.c_driver,
                c_filename.as_ptr(),
                size_x as c_int,
                size_y as c_int,
                bands as c_int,
                T::gdal_type(),
                null()
            ) };
        return match c_dataset.is_null() {
            true  => None,
            false => unsafe { Some(Dataset::_with_c_ptr(c_dataset)) },
        };
    }
}
