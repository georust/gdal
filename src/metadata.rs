use crate::errors::*;
use crate::gdal_major_object::MajorObject;
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string};
use gdal_sys::{self, CPLErr};
use std::ffi::CString;

pub trait Metadata: MajorObject {
    fn description(&self) -> Result<String> {
        let c_res = unsafe { gdal_sys::GDALGetDescription(self.gdal_object_ptr()) };
        if c_res.is_null() {
            return Err(_last_null_pointer_err("GDALGetDescription"));
        }
        Ok(_string(c_res))
    }

    fn metadata_domains(&self) -> Vec<String> {
        let mut domains = Vec::new();
        let c_res = unsafe { gdal_sys::GDALGetMetadataDomainList(self.gdal_object_ptr()) };

        if !c_res.is_null() {
            for i in 0.. {
                let p = unsafe { *c_res.offset(i) };
                if p.is_null() {
                    break;
                }

                domains.push(_string(p));
            }
        }
        unsafe { gdal_sys::CSLDestroy(c_res) };

        domains
    }

    fn metadata_domain(&self, domain: &str) -> Option<Vec<String>> {
        let mut metadata = Vec::new();
        if let Ok(c_domain) = CString::new(domain.to_owned()) {
            let c_res =
                unsafe { gdal_sys::GDALGetMetadata(self.gdal_object_ptr(), c_domain.as_ptr()) };

            if c_res.is_null() {
                return None;
            }

            if !c_res.is_null() {
                for i in 0.. {
                    let p = unsafe { *c_res.offset(i) };
                    if p.is_null() {
                        break;
                    }

                    metadata.push(_string(p));
                }
            }
        }

        Some(metadata)
    }

    fn metadata_item(&self, key: &str, domain: &str) -> Option<String> {
        if let Ok(c_key) = CString::new(key.to_owned()) {
            if let Ok(c_domain) = CString::new(domain.to_owned()) {
                let c_res = unsafe {
                    gdal_sys::GDALGetMetadataItem(
                        self.gdal_object_ptr(),
                        c_key.as_ptr(),
                        c_domain.as_ptr(),
                    )
                };
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
        let c_value = CString::new(value.to_owned())?;

        let c_res = unsafe {
            gdal_sys::GDALSetMetadataItem(
                self.gdal_object_ptr(),
                c_key.as_ptr(),
                c_value.as_ptr(),
                c_domain.as_ptr(),
            )
        };
        if c_res != CPLErr::CE_None {
            return Err(_last_cpl_err(c_res));
        }
        Ok(())
    }

    fn set_description(&mut self, description: &str) -> Result<()> {
        // For Datasets this sets the dataset name; normally
        // application code should not set the "description" for
        // GDALDatasets. For RasterBands it is actually a description
        // (if supported) or "".
        let c_description = CString::new(description.to_owned())?;
        unsafe { gdal_sys::GDALSetDescription(self.gdal_object_ptr(), c_description.as_ptr()) };
        Ok(())
    }
}
