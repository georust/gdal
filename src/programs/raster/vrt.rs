use std::{
    borrow::Borrow,
    ffi::{c_char, c_int, CString},
    path::Path,
    ptr::{null, null_mut},
};

use gdal_sys::GDALBuildVRTOptions;

use crate::{
    errors::*,
    utils::{_last_null_pointer_err, _path_to_c_string},
    Dataset,
};

/// Wraps a [GDALBuildVRTOptions] object.
///
/// [GDALBuildVRTOptions]: https://gdal.org/api/gdal_utils.html#_CPPv419GDALBuildVRTOptions
pub struct BuildVRTOptions {
    c_options: *mut GDALBuildVRTOptions,
}

impl BuildVRTOptions {
    /// See [GDALBuildVRTOptionsNew].
    ///
    /// [GDALBuildVRTOptionsNew]: https://gdal.org/api/gdal_utils.html#_CPPv422GDALBuildVRTOptionsNewPPcP28GDALBuildVRTOptionsForBinary
    pub fn new<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(args: I) -> Result<Self> {
        // Convert args to CStrings to add terminating null bytes
        let cstr_args = args
            .into_iter()
            .map(CString::new)
            .collect::<std::result::Result<Vec<_>, _>>()?;

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

    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_options(&self) -> *mut GDALBuildVRTOptions {
        self.c_options
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
    options: Option<BuildVRTOptions>,
) -> Result<Dataset> {
    _build_vrt(
        dest,
        &datasets
            .iter()
            .map(|x| x.borrow())
            .collect::<Vec<&Dataset>>(),
        options,
    )
}

fn _build_vrt(
    dest: Option<&Path>,
    datasets: &[&Dataset],
    options: Option<BuildVRTOptions>,
) -> Result<Dataset> {
    // Convert dest to CString
    let dest = dest.map(_path_to_c_string).transpose()?;
    let c_dest = dest.as_ref().map(|x| x.as_ptr()).unwrap_or(null());

    let c_options = options
        .as_ref()
        .map(|x| x.c_options as *const GDALBuildVRTOptions)
        .unwrap_or(null());

    let dataset_out = unsafe {
        // Get raw handles to the datasets
        let mut datasets_raw: Vec<gdal_sys::GDALDatasetH> =
            datasets.iter().map(|x| x.c_dataset()).collect();

        gdal_sys::GDALBuildVRT(
            c_dest,
            datasets_raw.len() as c_int,
            datasets_raw.as_mut_ptr(),
            null(),
            c_options,
            null_mut(),
        )
    };

    if dataset_out.is_null() {
        return Err(_last_null_pointer_err("GDALBuildVRT"));
    }

    let result = unsafe { Dataset::from_c_dataset(dataset_out) };

    Ok(result)
}
