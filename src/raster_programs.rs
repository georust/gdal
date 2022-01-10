use std::{borrow::Borrow, ptr::{null, null_mut}};
use std::ffi::CString;
use crate::{errors::*, Dataset};
use libc::{c_char, c_int};

/// Build a VRT from a list of datasets.
/// Wraps [GDALBuildVRT].
/// See the [program docs] for more details.
/// 
/// [GDALBuildVRT]: https://gdal.org/api/gdal_utils.html#gdal__utils_8h_1a057aaea8b0ed0476809a781ffa377ea4
/// [program docs]: https://gdal.org/programs/gdalbuildvrt.html
pub fn build_vrt<D: Borrow<Dataset>>(dest: Option<String>, datasets: &[D], args: Vec<String>) -> Result<Dataset> {
    // Convert dest to raw string
    let dest = match dest {
        Some(s) => CString::new(s)?.into_raw(),
        None => null_mut(),
    };

    // Convert args to raw strings
    let mut c_args = Vec::<*mut c_char>::new();
    for arg in args {
        c_args.push(CString::new(arg)?.into_raw());
    }
    c_args.push(null_mut()); // Null-terminate it

    let result = unsafe {
        let options = gdal_sys::GDALBuildVRTOptionsNew(c_args.as_mut_ptr(), null_mut());

        // Get raw handles to the datasets
        let mut datasets_raw: Vec<gdal_sys::GDALDatasetH> = datasets.iter().map(|x| x.borrow().c_dataset()).collect();

        let dataset_out = gdal_sys::GDALBuildVRT(dest, datasets_raw.len() as c_int, datasets_raw.as_mut_ptr(), null(), options, null_mut());

        gdal_sys::GDALBuildVRTOptionsFree(options);

        // Retake raw strings to free memory
        if !dest.is_null() {
            let _ = CString::from_raw(dest);
        }
        for c_arg in c_args {
            if !c_arg.is_null() {
                let _ = CString::from_raw(c_arg);
            }
        }

        if dataset_out.is_null() {
            return Err(GdalError::NullPointer{method_name: "GDALBuildVRT", msg: "".to_string()});
        }

        Dataset::from_c_dataset(dataset_out)
    };

    Ok(result)
}
