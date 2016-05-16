use libc::{c_char, c_void, c_int};
use utils::{_string};
use std::ffi::{CString};
use gdal_major_object::MajorObject;

#[link(name="gdal")]
extern {
    fn GDALGetMetadataItem(hGdalMayorObject: *const c_void, pszName: *const c_char, pszDomain: *const c_char) -> *const c_char;
    fn GDALSetMetadataItem(hGdalMayorObject: *const c_void, pszName: *const c_char, pszValue: *const c_char, pszDomain: *const c_char ) -> c_int;
}

pub trait Metadata: MajorObject {

    fn get_metadata_item(&self, key: &str, domain: &str) -> Option<String> {
        let c_key = CString::new(key.to_owned()).expect("Could not transform String to CString"); // FIXME: no unwrap
        let c_domain = CString::new(domain.to_owned()).expect("Could not transform String to CString");

        let c_res = unsafe { GDALGetMetadataItem(self.get_gdal_object_ptr(), c_key.as_ptr(), c_domain.as_ptr())};
        if c_res.is_null() {
            None
        }
        else {
            Some(_string(c_res))
        }
    }

    fn set_metadata_item(&mut self, key: &str, value: &str, domain: &str) -> Result<(), ()> {
        let c_key = CString::new(key.to_owned()).expect("Could not transform String to CString"); // FIXME: no unwrap
        let c_domain = CString::new(domain.to_owned()).expect("Could not transform String to CString");
        let c_value =  CString::new(value.to_owned()).expect("Could not transform String to CString");

        let c_res = unsafe { GDALSetMetadataItem(self.get_gdal_object_ptr(), c_key.as_ptr(), c_value.as_ptr(), c_domain.as_ptr())};
        if c_res == 0 { // TODO: convert into CPLErr?
            Ok(())
        }
        else {
            Err(())
        }
    }

}
