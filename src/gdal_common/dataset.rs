use gdal_sys::{self, GDALAccess, GDALDatasetH, GDALMajorObjectH};
use super::gdal_major_object::MajorObject;

use std::{path::Path, ffi::CString, ptr};

use crate::{utils::{_string, _last_null_pointer_err}, errors::*, _register_drivers, DriverCommon, Driver, Metadata};

pub struct Dataset {
    c_dataset: GDALDatasetH,
}

impl Dataset {
    pub unsafe fn from_c_dataset(c_dataset: GDALDatasetH) -> Dataset {
        Dataset {
            c_dataset
        }
    }
}

impl MajorObject for Dataset {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_dataset
    }
}

impl Metadata for Dataset {}

impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::GDALClose(self.c_dataset);
        }
    }
}

impl AsRef<Dataset> for Dataset {
    fn as_ref(&self) -> &Dataset {
        self
    }    
}

pub trait DatasetCommon: AsRef<Dataset> {
    fn c_dataset(&self) -> GDALDatasetH;

    fn open(path: &Path) -> Result<Dataset> {
        Self::open_ex(path, None, None, None, None)
    }

    // TODO: use the parameters
    fn open_ex(path: &Path, open_flags: Option<u32>, allowed_drivers: Option<&str>, open_options: Option<&str>, sibling_files: Option<&str>) -> Result<Dataset> {
        _register_drivers();
        let filename = path.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_open_flags = open_flags.unwrap_or(GDALAccess::GA_ReadOnly); // This defaults to GdalAccess::GA_ReadOnly
        

        let c_dataset = unsafe { gdal_sys::GDALOpenEx(c_filename.as_ptr(), c_open_flags, ptr::null(), ptr::null(), ptr::null()) };
        if c_dataset.is_null() {
            Err(_last_null_pointer_err("GDALOpenEx"))?;
        }
        Ok(Dataset { c_dataset })
    }

    unsafe fn from_c_ptr(c_dataset: GDALDatasetH) -> Dataset {
        Dataset { c_dataset }
    }

    fn projection(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetProjectionRef(self.c_dataset()) };
        _string(rv)
    }

    fn set_projection(&self, projection: &str) -> Result<()> {
        let c_projection = CString::new(projection)?;
        unsafe { gdal_sys::GDALSetProjection(self.c_dataset(), c_projection.as_ptr()) };
        Ok(())
    }

    fn create_copy(&self, driver: &Driver, filename: &str) -> Result<Dataset> {
        let c_filename = CString::new(filename)?;
        let c_dataset = unsafe {
            gdal_sys::GDALCreateCopy(
                driver.c_driver(),
                c_filename.as_ptr(),
                self.c_dataset(),
                0,
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            )
        };
        if c_dataset.is_null() {
            Err(_last_null_pointer_err("GDALCreateCopy"))?;
        }
        Ok(unsafe {Dataset::from_c_dataset(c_dataset)})
    }


    fn driver(&self) -> Driver {
        unsafe {
            let c_driver = gdal_sys::GDALGetDatasetDriver(self.c_dataset());
            Driver::from_c_driver(c_driver)
        }
    }

}

impl DatasetCommon for Dataset {
    fn c_dataset(&self) -> GDALDatasetH {
        self.c_dataset
    }    
}