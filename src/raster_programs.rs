use crate::{errors::*, utils::_last_null_pointer_err, Dataset};
use gdal_sys::GDALBuildVRTOptions;
use libc::{c_char, c_int};
use std::{
    borrow::Borrow,
    ffi::CString,
    path::Path,
    ptr::{null, null_mut},
};

/// Wraps a [GDALBuildVRTOptions] object.
///
/// [GDALBuildVRTOptions]: https://gdal.org/api/gdal_utils.html#_CPPv419GDALBuildVRTOptions
struct BuildVRTOptions {
    c_options: *mut GDALBuildVRTOptions,
}

impl BuildVRTOptions {
    /// See [GDALBuildVRTOptionsNew].
    ///
    /// [GDALBuildVRTOptionsNew]: https://gdal.org/api/gdal_utils.html#_CPPv422GDALBuildVRTOptionsNewPPcP28GDALBuildVRTOptionsForBinary
    fn new(args: Vec<String>) -> Result<Self> {
        // Convert args to CStrings to add terminating null bytes
        let mut cstr_args = Vec::<CString>::new();
        for arg in args {
            cstr_args.push(CString::new(arg)?);
        }
        // Get pointers to the strings
        // These strings don't actually get modified, the C API is just not const-correct
        // Null-terminate the list
        let mut c_args = cstr_args
            .iter()
            .map(|x| x.as_ptr() as *mut c_char)
            .chain(std::iter::once(null_mut()))
            .collect::<Vec<_>>();
        unsafe {
            Ok(Self {
                c_options: gdal_sys::GDALBuildVRTOptionsNew(c_args.as_mut_ptr(), null_mut()),
            })
        }
    }
}

impl Drop for BuildVRTOptions {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::GDALBuildVRTOptionsFree(self.c_options);
        }
    }
}

/// Build a VRT from a list of datasets.
/// Wraps [GDALBuildVRT].
/// See the [program docs] for more details.
///
/// [GDALBuildVRT]: https://gdal.org/api/gdal_utils.html#gdal__utils_8h_1a057aaea8b0ed0476809a781ffa377ea4
/// [program docs]: https://gdal.org/programs/gdalbuildvrt.html
pub fn build_vrt<D: Borrow<Dataset>>(
    dest: Option<&Path>,
    datasets: &[D],
    args: Vec<String>,
) -> Result<Dataset> {
    _build_vrt(
        dest,
        &datasets
            .iter()
            .map(|x| x.borrow())
            .collect::<Vec<&Dataset>>(),
        args,
    )
}

fn _build_vrt(dest: Option<&Path>, datasets: &[&Dataset], args: Vec<String>) -> Result<Dataset> {
    // Convert dest to CString
    let dest = match dest {
        Some(x) => Some(CString::new(x.to_string_lossy().to_string())?),
        None => None,
    };
    let c_dest = match dest {
        Some(x) => x.as_ptr(),
        None => null(),
    };

    let result = unsafe {
        let options = BuildVRTOptions::new(args)?;
        // Get raw handles to the datasets
        let mut datasets_raw: Vec<gdal_sys::GDALDatasetH> =
            datasets.iter().map(|x| x.c_dataset()).collect();

        let dataset_out = gdal_sys::GDALBuildVRT(
            c_dest,
            datasets_raw.len() as c_int,
            datasets_raw.as_mut_ptr(),
            null(),
            options.c_options,
            null_mut(),
        );

        if dataset_out.is_null() {
            return Err(_last_null_pointer_err("GDALBuildVRT"));
        }

        Dataset::from_c_dataset(dataset_out)
    };

    Ok(result)
}
