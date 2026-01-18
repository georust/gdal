use std::borrow::Borrow;
use std::ffi::CString;
use std::ptr::{null, null_mut};

use crate::{
    errors::*,
    utils::{_last_null_pointer_err},
    Dataset,
};
use gdal_sys::{GDALVectorTranslate, GDALVectorTranslateOptions, GDALVectorTranslateOptionsFree};
use libc::c_char;
use crate::programs::destination::DatasetDestination;

/// Wraps a [GDALVectorTranslateOptions] object.
///
/// [GDALVectorTranslateOptions]: https://gdal.org/api/gdal_utils.html#_CPPv426GDALVectorTranslateOptions
pub struct VectorTranslateOptions{
    c_options: *mut GDALVectorTranslateOptions
}

impl VectorTranslateOptions{
    /// See [GDALVectorTranslateOptionsNew].
    ///
    /// [GDALVectorTranslateOptionsNew]: https://gdal.org/api/gdal_utils.html#_CPPv429GDALVectorTranslateOptionsNewPPcP35GDALVectorTranslateOptionsForBinary
    pub fn new<S:Into<Vec<u8>>,I:IntoIterator<Item=S>>(args:I)->Result<Self>{
        let cstr_args = args
            .into_iter()
            .map(CString::new)
            .collect::<std::result::Result<Vec<_>,_>>()?;
        let mut c_args = cstr_args
            .iter()
            .map(|x| x.as_ptr() as *mut c_char)
            .chain(std::iter::once(null_mut()))
            .collect::<Vec<_>>();

        unsafe {
            Ok(Self {
                c_options: gdal_sys::GDALVectorTranslateOptionsNew(c_args.as_mut_ptr(), null_mut()),
            })
        }
    }
    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_options(&self) -> *mut GDALVectorTranslateOptions {
        self.c_options
    }
}

impl Drop for VectorTranslateOptions {
    fn drop(&mut self) {
        unsafe {
            GDALVectorTranslateOptionsFree(self.c_options);
        }
    }
}

impl TryFrom<Vec<&str>> for VectorTranslateOptions {
    type Error = GdalError;

    fn try_from(value: Vec<&str>) -> Result<Self> {
        VectorTranslateOptions::new(value)
    }
}



/// Converts simple features data between file formats.
///
/// Wraps [GDALVectorTranslate].
/// See the [program docs] for more details.
///
/// [GDALVectorTranslate]: https://gdal.org/api/gdal_utils.html#_CPPv419GDALVectorTranslatePKc12GDALDatasetHiP12GDALDatasetHPK26GDALVectorTranslateOptionsPi
/// [program docs]: https://gdal.org/programs/ogr2ogr.html
///
pub fn vector_translate<D:Borrow<Dataset>>(src: &[D], dest: DatasetDestination, options: Option<VectorTranslateOptions>) ->Result<Dataset> {
    _vector_translate(
        &src
            .iter()
            .map(|x|x.borrow())
            .collect::<Vec<&Dataset>>()
        ,dest,
        options
    )
}

fn _vector_translate(datasets: &[&Dataset], mut dest: DatasetDestination,options:Option<VectorTranslateOptions>)->Result<Dataset>{

    let (psz_dest_option, h_dst_ds) = match &dest {
        DatasetDestination::Path(c_path) => (Some(c_path), null_mut()),
        DatasetDestination::Dataset { dataset, .. } => (None, dataset.c_dataset()),
    };

    let psz_dest = psz_dest_option.map(|x| x.as_ptr()).unwrap_or_else(null);

    let c_options = options
        .as_ref()
        .map(|x| x.c_options as *const GDALVectorTranslateOptions)
        .unwrap_or(null());

    let dataset_out = unsafe {
        // Get raw handles to the datasets
        let mut datasets_raw: Vec<gdal_sys::GDALDatasetH> =
            datasets.iter().map(|x| x.c_dataset()).collect();

        let data = GDALVectorTranslate(
            psz_dest,
            h_dst_ds,
            // only 1 supported currently
            1,
            datasets_raw.as_mut_ptr(),
            c_options,
            null_mut(),
        );

        // GDAL takes the ownership of `h_dst_ds`
        dest.do_no_drop_dataset();

        data

    };

    if dataset_out.is_null() {
        return Err(_last_null_pointer_err("GDALVectorTranslate"));
    }

    let result = unsafe { Dataset::from_c_dataset(dataset_out) };

    Ok(result)
}




#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::Dataset;
    use crate::programs::vector::vector_translate::vector_translate;

    #[test]
    fn test_vector_translate(){
        let path = "fixtures/roads.geojson";
        let dataset = &Dataset::open(Path::new(path)).unwrap();
        let out = "fixtures/roads.sql";
        let dest = out.try_into().unwrap();
        let dst = vector_translate(&[dataset], dest, None);
    }
}
