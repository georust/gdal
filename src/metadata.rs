use utils::{_string};
use std::ffi::{CString};
use gdal_major_object::MajorObject;
use utils::{_last_cpl_err};
use errors::*;
use gdal_sys::{gdal, cpl_error};

pub trait Metadata: MajorObject {

    fn description(&self) -> Result<String>{
        let c_res = unsafe { gdal::GDALGetDescription(self.gdal_object_ptr())};
        if c_res.is_null() {
            return Err(ErrorKind::NullPointer("GDALGetDescription").into());
        }
        Ok(_string(c_res))
    }

    fn metadata_item(&self, key: &str, domain: &str) -> Option<String> {
        if let Ok(c_key) = CString::new(key.to_owned()) {
            if let Ok(c_domain) = CString::new(domain.to_owned()){
                let c_res = unsafe { gdal::GDALGetMetadataItem(self.gdal_object_ptr(), c_key.as_ptr(), c_domain.as_ptr())};
                if !c_res.is_null() {
                    return Some(_string(c_res));
                }
            }
        }
        None
    }

    fn set_metadata_item(&mut self, key: &str, value: &str, domain: &str) -> Result<()> {
        let c_key = CString::new(key.to_owned())?;
        let c_domain = CString::new(domain.to_owned())?;
        let c_value =  CString::new(value.to_owned())?;
        
        let c_res = unsafe { gdal::GDALSetMetadataItem(self.gdal_object_ptr(), c_key.as_ptr(), c_value.as_ptr(), c_domain.as_ptr())};
        if c_res != cpl_error::CPLErr::CE_None {
            return Err(_last_cpl_err(c_res).into());
        }
        Ok(())
    }

}
