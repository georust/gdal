use super::GdalType;
use crate::errors::*;
use crate::utils::{_last_cpl_err, _string, _string_array};
use crate::{cpl::CslStringList, Dataset};
use gdal_sys::{
    GDALDimensionGetName, GDALDimensionGetSize, GDALDimensionHS, GDALGroupGetMDArrayNames,
    GDALGroupGetName, GDALGroupH, GDALGroupOpenMDArray, GDALMDArrayGetDataType,
    GDALMDArrayGetDimensionCount, GDALMDArrayGetDimensions, GDALMDArrayGetTotalElementsCount,
    GDALMDArrayHS,
};
use libc::c_void;
use std::convert::TryInto;
use std::ffi::CString;

#[cfg(feature = "ndarray")]
use ndarray::{ArrayD, IxDyn};
use std::fmt::Debug;

#[cfg(test)]
mod tests {
    use crate::{cpl, Dataset, DatasetOptions, GdalOpenFlags};

    #[test]
    fn test_root_group_name() {
        let options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", options).unwrap();
        let root_group = dataset.root_group().unwrap();
        let root_group_name = root_group.name();
        assert_eq!(root_group_name, "/");
    }
    #[test]
    fn test_array_names() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();
        let root_group = dataset.root_group().unwrap();
        let options = cpl::CslStringList::new(); //Driver specific options determining how groups should be retrieved. Pass nullptr for default behavior.
        let array_names = root_group.array_names(options);
        assert_eq!(array_names, vec!["Band1".to_string()])
    }

    #[test]
    fn test_n_dimension() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();
        let root_group = dataset.root_group().unwrap();
        let array_name = "Band1".to_string();
        let options = cpl::CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.md_array(array_name, options);
        let n_dimension = md_array.n_dimension();
        assert_eq!(2, n_dimension);
    }

    #[test]
    fn test_n_elements() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();
        let root_group = dataset.root_group().unwrap();
        let array_name = "Band1".to_string();
        let options = cpl::CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.md_array(array_name, options);
        let n_elements = md_array.n_elements();
        assert_eq!(400, n_elements);
    }

    #[test]
    fn test_dimension_name() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();
        let root_group = dataset.root_group().unwrap();
        let array_name = "Band1".to_string();
        let options = cpl::CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.md_array(array_name, options);
        let dimensions = md_array.get_dimensions().unwrap();
        let mut dimension_names = Vec::new();
        for dimension in dimensions {
            dimension_names.push(dimension.name());
        }
        assert_eq!(dimension_names, vec!["y".to_string(), "x".to_string()])
    }
    #[test]
    fn test_dimension_size() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();
        let root_group = dataset.root_group().unwrap();
        let array_name = "Band1".to_string();
        let options = cpl::CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.md_array(array_name, options);
        let dimensions = md_array.get_dimensions().unwrap();
        let mut dimensions_size = Vec::new();
        for dimension in dimensions {
            dimensions_size.push(dimension.size());
        }
        assert_eq!(dimensions_size, vec![20, 20])
    }
}

/// Represent an MDArray in a Group
///
/// This object carries the lifetime of the Group that
/// contains it. This is necessary to prevent the Group
/// from being dropped before the mdarray.
#[derive(Debug)]
pub struct MDArray<'a> {
    c_mdarray: *mut GDALMDArrayHS, //H
    group: &'a Group<'a>,
}

#[allow(dead_code)]
pub struct Dimension<'a> {
    c_dimension: *mut GDALDimensionHS,
    md_array: &'a MDArray<'a>,
}

impl<'a> Dimension<'a> {
    /// Create a MDArray from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub fn from_c_dimension(md_array: &'a MDArray<'a>, c_dimension: *mut GDALDimensionHS) -> Self {
        Dimension {
            c_dimension: (c_dimension),
            md_array: (md_array),
        }
    }
    pub fn size(self) -> usize {
        unsafe { GDALDimensionGetSize(self.c_dimension) as usize }
    }

    pub fn name(self) -> String {
        _string(unsafe { GDALDimensionGetName(self.c_dimension) })
    }
}

impl<'a> MDArray<'a> {
    /// Create a MDArray from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub fn from_c_mdarray(group: &'a Group, c_mdarray: *mut GDALMDArrayHS) -> Self {
        MDArray { c_mdarray, group }
    }

    pub fn n_dimension(&self) -> usize {
        unsafe { GDALMDArrayGetDimensionCount(self.c_mdarray) }
    }

    pub fn n_elements(&self) -> u64 {
        unsafe { GDALMDArrayGetTotalElementsCount(self.c_mdarray) }
    }

    pub fn get_dimensions(&self) -> Result<Vec<Dimension>> {
        // break on ndims not is_null
        let n_dimension = self.n_dimension();
        unsafe {
            let mut pn_count: usize = 0;
            let pn_count_ptr: *mut usize = &mut pn_count;
            let c_dimensions = GDALMDArrayGetDimensions(self.c_mdarray, pn_count_ptr);
            // if c_group.is_null() {
            //     return Err(_last_null_pointer_err("GDALGetRasterBand"));
            // }
            let mut dimensions: Vec<Dimension> = Vec::new();
            let mut i = 0;
            while i < n_dimension {
                let ptr = c_dimensions.add(i);
                let next = ptr.read();
                let value = Dimension::from_c_dimension(self, next);
                i += 1;
                dimensions.push(value);
            }
            Ok(dimensions)
        }
    }

    pub fn read_into_slice<T: Copy + GdalType>(
        &self,
        buffer: &mut [T],
        array_start_index: Vec<u64>,
        count: Vec<usize>,
    ) -> Result<()> {
        // let array_start_index = [array_start_index.0, array_start_index.1];
        // let count =  [count.0, count.1];
        let array_step: *const i64 = std::ptr::null();
        let buffer_stride: *const i64 = std::ptr::null();
        let data_type = unsafe { GDALMDArrayGetDataType(self.c_mdarray) };
        let p_dst_buffer_alloc_start: *mut libc::c_void = std::ptr::null_mut();
        let n_dst_buffer_alloc_size = 0;

        let rv = unsafe {
            gdal_sys::GDALMDArrayRead(
                self.c_mdarray,
                array_start_index.as_ptr(),
                count.as_ptr(),
                array_step,
                buffer_stride,
                data_type,
                buffer.as_mut_ptr() as *mut c_void, // pDstBuffer: *mut libc::c_void,
                p_dst_buffer_alloc_start,           // pDstBufferAllocStart: *const libc::c_void,
                n_dst_buffer_alloc_size,
            )
        };

        if rv != 1 {
            return Err(_last_cpl_err(rv.try_into().unwrap())); // this is probably incorrect!
        }

        Ok(())
    }

    /// Read a 'Buffer<T>' from this band. T implements 'GdalType'
    ///
    /// # Arguments
    /// * array_start_index - Values representing the starting index to read in each dimension (in [0, aoDims[i].GetSize()-1] range). Array of GetDimensionCount() values. Must not be nullptr, unless for a zero-dimensional array.
    /// * count - Values representing the number of values to extract in each dimension. Array of GetDimensionCount() values. Must not be nullptr, unless for a zero-dimensional array.
    pub fn read_as<T: Copy + GdalType>(
        &self,
        array_start_index: Vec<u64>,
        count: Vec<usize>,
    ) -> Result<Vec<T>> {
        let pixels: usize = count.iter().product();
        let mut data: Vec<T> = Vec::with_capacity(pixels);

        // Safety: the read_into_slice line below writes
        // exactly pixel elements into the slice, before we
        // read from this slice. This paradigm is suggested
        // in the rust std docs
        // (https://doc.rust-lang.org/std/vec/struct.Vec.html#examples-18)
        unsafe {
            data.set_len(pixels);
        };
        self.read_into_slice(&mut data, array_start_index, count)?;

        Ok(data)
    }

    #[cfg(feature = "ndarray")]
    /// Read a 'Array2<T>' from this band. T implements 'GdalType'.
    ///
    /// # Arguments
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != array_size)
    /// * array_size - the desired size of the 'Array'
    /// * e_resample_alg - the resample algorithm used for the interpolation
    /// # Docs
    /// The Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis).
    pub fn read_as_array<T: Copy + GdalType + Debug>(
        &self,
        array_start_index: Vec<u64>,
        count: Vec<usize>,
        array_size: Vec<usize>,
    ) -> Result<ArrayD<T>> {
        let data = self.read_as::<T>(array_start_index, count)?;
        // Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis)

        let dim: IxDyn = IxDyn(&array_size);
        Ok(ArrayD::from_shape_vec(dim, data)?)
    }
}

/// Represent a mdarray in a dataset
///
/// This object carries the lifetime of the dataset that
/// contains it. This is necessary to prevent the dataset
/// from being dropped before the group.
#[derive(Debug)]
pub struct Group<'a> {
    c_group: GDALGroupH,
    dataset: &'a Dataset,
}

impl<'a> Group<'a> {
    /// Create a Group from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_group(dataset: &'a Dataset, c_group: GDALGroupH) -> Self {
        Group { c_group, dataset }
    }
    pub fn name(&self) -> String {
        _string(unsafe { GDALGroupGetName(self.c_group) })
    }

    pub fn array_names(&self, options: CslStringList) -> Vec<String> {
        let options = options.as_ptr();
        let c_array_names = unsafe { GDALGroupGetMDArrayNames(self.c_group, options) };
        _string_array(c_array_names)
    }

    pub fn md_array(&self, name: String, options: CslStringList) -> MDArray {
        let name = CString::new(name).unwrap();
        let c_mdarray =
            unsafe { GDALGroupOpenMDArray(self.c_group, name.as_ptr(), options.as_ptr()) };

        MDArray::from_c_mdarray(self, c_mdarray)
    }
    // pub unsafe fn array(&self, array_name: String, options: CslStringList) -> MDArray{
    //     let name = CString::new(array_name).unwrap();
    //     let options = options.as_ptr();
    //     let array_h =  GDALGroupOpenMDArray(self.c_group, name.as_ptr(), options);
    //     MDArray::from_c_mdarray(&self, array_h)
    // }
}
