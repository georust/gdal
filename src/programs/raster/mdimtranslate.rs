use crate::{
    errors::*,
    utils::{_last_null_pointer_err, _path_to_c_string},
    Dataset,
};
use gdal_sys::{GDALMultiDimTranslate, GDALMultiDimTranslateOptions};
use libc::{c_char, c_int};
use std::{
    borrow::Borrow,
    ffi::CString,
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    ptr::{null, null_mut},
};

use crate::programs::destination::DatasetDestination;

type MultiDimTranslateDestination = DatasetDestination;

/// Wraps a [GDALMultiDimTranslateOptions] object.
///
/// [GDALMultiDimTranslateOptions]: https://gdal.org/api/gdal_utils.html#_CPPv428GDALMultiDimTranslateOptions
///
pub struct MultiDimTranslateOptions {
    c_options: *mut GDALMultiDimTranslateOptions,
}

impl MultiDimTranslateOptions {
    /// See [GDALMultiDimTranslateOptionsNew].
    ///
    /// [GDALMultiDimTranslateOptionsNew]: https://gdal.org/api/gdal_utils.html#_CPPv431GDALMultiDimTranslateOptionsNewPPcP37GDALMultiDimTranslateOptionsForBinary
    ///
    pub fn new<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(args: I) -> Result<Self> {
        // Convert args to CStrings to add terminating null bytes
        let cstr_args = args
            .into_iter()
            .map(CString::new)
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Self::_new(&cstr_args)
    }

    fn _new(cstr_args: &[CString]) -> Result<Self> {
        // Get pointers to the strings
        let mut c_args = cstr_args
            .iter()
            .map(|x| x.as_ptr() as *mut c_char) // These strings don't actually get modified, the C API is just not const-correct
            .chain(std::iter::once(null_mut())) // Null-terminate the list
            .collect::<Vec<_>>();

        unsafe {
            Ok(Self {
                c_options: gdal_sys::GDALMultiDimTranslateOptionsNew(
                    c_args.as_mut_ptr(),
                    null_mut(),
                ),
            })
        }
    }

    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    ///
    pub unsafe fn c_options(&self) -> *mut GDALMultiDimTranslateOptions {
        self.c_options
    }
}

impl Drop for MultiDimTranslateOptions {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::GDALMultiDimTranslateOptionsFree(self.c_options);
        }
    }
}

impl TryFrom<Vec<&str>> for MultiDimTranslateOptions {
    type Error = GdalError;

    fn try_from(value: Vec<&str>) -> Result<Self> {
        MultiDimTranslateOptions::new(value)
    }
}


/// Converts raster data between different formats.
///
/// Wraps [GDALMultiDimTranslate].
/// See the [program docs] for more details.
///
/// [GDALMultiDimTranslate]: https://gdal.org/api/gdal_utils.html#_CPPv421GDALMultiDimTranslatePKc12GDALDatasetHiP12GDALDatasetHPK28GDALMultiDimTranslateOptionsPi
/// [program docs]: https://gdal.org/programs/gdalmdimtranslate.html
///
pub fn multi_dim_translate<D: Borrow<Dataset>>(
    input: &[D],
    destination: MultiDimTranslateDestination,
    options: Option<MultiDimTranslateOptions>,
) -> Result<Dataset> {
    _multi_dim_translate(
        &input.iter().map(|x| x.borrow()).collect::<Vec<&Dataset>>(),
        destination,
        options,
    )
}

fn _multi_dim_translate(
    input: &[&Dataset],
    mut destination: MultiDimTranslateDestination,
    options: Option<MultiDimTranslateOptions>,
) -> Result<Dataset> {
    let (psz_dest_option, h_dst_ds) = match &destination {
        MultiDimTranslateDestination::Path(c_path) => (Some(c_path), null_mut()),
        MultiDimTranslateDestination::Dataset { dataset, .. } => (None, dataset.c_dataset()),
    };

    let psz_dest = psz_dest_option.map(|x| x.as_ptr()).unwrap_or_else(null);

    let mut pah_src_ds: Vec<gdal_sys::GDALDatasetH> = input.iter().map(|x| x.c_dataset()).collect();

    let ps_options = options
        .as_ref()
        .map(|x| x.c_options as *const GDALMultiDimTranslateOptions)
        .unwrap_or(null());

    let mut pb_usage_error: c_int = 0;

    let dataset_out = unsafe {
        let data = GDALMultiDimTranslate(
            psz_dest,
            h_dst_ds,
            pah_src_ds.len() as c_int,
            pah_src_ds.as_mut_ptr(),
            ps_options,
            &mut pb_usage_error as *mut c_int,
        );

        // GDAL takes the ownership of `h_dst_ds`
        destination.do_no_drop_dataset();

        data
    };

    if dataset_out.is_null() {
        return Err(_last_null_pointer_err("GDALMultiDimTranslate"));
    }

    let result = unsafe { Dataset::from_c_dataset(dataset_out) };

    Ok(result)
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::options::DatasetOptions;
    use crate::{DriverManager, GdalOpenFlags};

    #[test]
    #[cfg_attr(not(all(major_ge_3, minor_ge_4)), ignore)]
    fn test_build_tiff_from_path() {
        let fixture = "/vsizip/fixtures/cf_nasa_4326.zarr.zip";

        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex(fixture, dataset_options).unwrap();
        let mem_file_path = "/vsimem/2d3e9124-a7a0-413e-97b5-e79d46e50ff8";

        let dataset = multi_dim_translate(
            &[dataset],
            mem_file_path.try_into().unwrap(),
            Some(
                vec![
                    "-array",
                    "name=/science/grids/imagingGeometry/lookAngle,view=[2,:,:]",
                ]
                .try_into()
                .unwrap(),
            ),
        )
        .unwrap();

        assert_eq!(dataset.raster_size(), (5, 7));
        assert_eq!(dataset.raster_count(), 1);
    }

    #[test]
    #[cfg_attr(not(all(major_ge_3, minor_ge_4)), ignore)]
    fn test_build_tiff_from_dataset() {
        let fixture = "/vsizip/fixtures/cf_nasa_4326.zarr.zip";

        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex(fixture, dataset_options).unwrap();

        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let output_dataset = driver.create("", 5, 7, 1).unwrap();

        let error = multi_dim_translate(
            &[output_dataset],
            dataset.into(),
            Some(
                MultiDimTranslateOptions::new(vec![
                    "-array",
                    "name=/science/grids/imagingGeometry/lookAngle,view=[2,:,:]",
                ])
                .unwrap(),
            ),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "GDAL method 'GDALMultiDimTranslate' returned a NULL pointer. Error msg: 'Update of existing file not supported yet'"
        );
    }
}
