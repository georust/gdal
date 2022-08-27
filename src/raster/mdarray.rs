use super::GdalType;
use crate::errors::*;
use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string, _string_array};
use crate::{cpl::CslStringList, Dataset};
use gdal_sys::{
    CPLErr, CSLDestroy, GDALAttributeGetDataType, GDALAttributeGetDimensionsSize, GDALAttributeH,
    GDALAttributeReadAsDouble, GDALAttributeReadAsDoubleArray, GDALAttributeReadAsInt,
    GDALAttributeReadAsIntArray, GDALAttributeReadAsString, GDALAttributeReadAsStringArray,
    GDALAttributeRelease, GDALDataType, GDALDimensionGetIndexingVariable, GDALDimensionGetName,
    GDALDimensionGetSize, GDALDimensionHS, GDALDimensionRelease, GDALExtendedDataTypeClass,
    GDALExtendedDataTypeGetClass, GDALExtendedDataTypeGetNumericDataType, GDALExtendedDataTypeH,
    GDALExtendedDataTypeRelease, GDALGroupGetAttribute, GDALGroupGetDimensions,
    GDALGroupGetGroupNames, GDALGroupGetMDArrayNames, GDALGroupGetName, GDALGroupH,
    GDALGroupOpenGroup, GDALGroupOpenMDArray, GDALGroupRelease, GDALMDArrayGetAttribute,
    GDALMDArrayGetDataType, GDALMDArrayGetDimensionCount, GDALMDArrayGetDimensions,
    GDALMDArrayGetNoDataValueAsDouble, GDALMDArrayGetSpatialRef, GDALMDArrayGetTotalElementsCount,
    GDALMDArrayGetUnit, GDALMDArrayH, GDALMDArrayRelease, OSRDestroySpatialReference, VSIFree,
};
use libc::c_void;
use std::ffi::CString;
use std::os::raw::c_char;

#[cfg(feature = "ndarray")]
use ndarray::{ArrayD, IxDyn};
use std::fmt::Debug;

/// Represent an MDArray in a Group
///
/// This object carries the lifetime of the Group that
/// contains it. This is necessary to prevent the Group
/// from being dropped before the mdarray.
#[derive(Debug)]
pub struct MDArray<'a> {
    c_mdarray: GDALMDArrayH,
    _parent: GroupOrDimension<'a>,
}

#[derive(Debug)]
pub enum GroupOrDimension<'a> {
    Group { _group: &'a Group<'a> },
    Dimension { _dimension: &'a Dimension<'a> },
}

#[derive(Debug)]
pub enum GroupOrArray<'a> {
    Group { _group: &'a Group<'a> },
    MDArray { _md_array: &'a MDArray<'a> },
}

impl Drop for MDArray<'_> {
    fn drop(&mut self) {
        unsafe {
            GDALMDArrayRelease(self.c_mdarray);
        }
    }
}

impl<'a> MDArray<'a> {
    /// Create a MDArray from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_mdarray_and_group(_group: &'a Group, c_mdarray: GDALMDArrayH) -> Self {
        Self {
            c_mdarray,
            _parent: GroupOrDimension::Group { _group },
        }
    }

    /// Create a MDArray from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_mdarray_and_dimension(
        _dimension: &'a Dimension,
        c_mdarray: GDALMDArrayH,
    ) -> Self {
        Self {
            c_mdarray,
            _parent: GroupOrDimension::Dimension { _dimension },
        }
    }

    pub fn num_dimensions(&self) -> usize {
        unsafe { GDALMDArrayGetDimensionCount(self.c_mdarray) }
    }

    pub fn num_elements(&self) -> u64 {
        unsafe { GDALMDArrayGetTotalElementsCount(self.c_mdarray) }
    }

    pub fn dimensions(&self) -> Result<Vec<Dimension>> {
        unsafe {
            let mut num_dimensions: usize = 0;
            let c_dimensions =
                GDALMDArrayGetDimensions(self.c_mdarray, std::ptr::addr_of_mut!(num_dimensions));

            // `num_dimensions` is `0`, we can safely return an empty vector
            // `GDALMDArrayGetDimensions` does not state that errors can occur
            if num_dimensions > 0 && c_dimensions.is_null() {
                return Err(_last_null_pointer_err("GDALMDArrayGetDimensions"));
            }

            let dimensions_ref = std::slice::from_raw_parts_mut(c_dimensions, num_dimensions);

            let mut dimensions: Vec<Dimension> = Vec::with_capacity(num_dimensions);

            for c_dimension in dimensions_ref {
                let dimension = Dimension::from_c_dimension(
                    GroupOrArray::MDArray { _md_array: self },
                    *c_dimension,
                );
                dimensions.push(dimension);
            }

            // only free the array, not the dimensions themselves
            VSIFree(c_dimensions as *mut c_void);

            Ok(dimensions)
        }
    }

    pub fn datatype(&self) -> ExtendedDataType {
        unsafe {
            let c_data_type = GDALMDArrayGetDataType(self.c_mdarray);

            ExtendedDataType::from_c_extended_data_type(c_data_type)
        }
    }

    /// Wrapper for `GDALMDArrayRead`
    ///
    /// # Params
    /// * buffer - Mutable buffer to read into
    /// * array_start_index - Values representing the starting index to read in each dimension (in [0, aoDims[i].GetSize()-1] range).
    ///   Array of GetDimensionCount() values. Must not be empty, unless for a zero-dimensional array.
    /// * count - Values representing the number of values to extract in each dimension. Array of GetDimensionCount() values.
    ///   Must not be empty, unless for a zero-dimensional array.
    ///
    pub fn read_into_slice<T: Copy + GdalType>(
        &self,
        buffer: &mut [T],
        array_start_index: Vec<u64>,
        count: Vec<usize>,
    ) -> Result<()> {
        // If set to nullptr, [1, 1, … 1] will be used as a default to indicate consecutive elements.
        let array_step: *const i64 = std::ptr::null();
        // If set to nullptr, will be set so that pDstBuffer is written in a compact way,
        // with elements of the last / fastest varying dimension being consecutive.
        let buffer_stride: *const i64 = std::ptr::null();
        let p_dst_buffer_alloc_start: *mut libc::c_void = std::ptr::null_mut();
        let n_dst_buffer_alloc_size = 0;

        let rv = unsafe {
            let data_type = GDALMDArrayGetDataType(self.c_mdarray);

            let rv = gdal_sys::GDALMDArrayRead(
                self.c_mdarray,
                array_start_index.as_ptr(),
                count.as_ptr(),
                array_step,
                buffer_stride,
                data_type,
                buffer.as_mut_ptr() as *mut c_void,
                p_dst_buffer_alloc_start,
                n_dst_buffer_alloc_size,
            );

            GDALExtendedDataTypeRelease(data_type);

            rv
        };

        // `rv` is boolean
        if rv != 1 {
            // OSGeo Python wrapper treats it as `CE_Failure`
            return Err(_last_cpl_err(CPLErr::CE_Failure));
        }

        Ok(())
    }

    /// Read a 'Buffer<T>' from this band. T implements 'GdalType'
    ///
    /// # Arguments
    /// * array_start_index - Values representing the starting index to read in each dimension (in [0, aoDims[i].GetSize()-1] range).
    ///   Array of GetDimensionCount() values. Must not be empty, unless for a zero-dimensional array.
    /// * count - Values representing the number of values to extract in each dimension. Array of GetDimensionCount() values.
    ///   Must not be empty, unless for a zero-dimensional array.
    ///
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
            self.read_into_slice(&mut data, array_start_index, count)?;
            data.set_len(pixels);
        };

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

    /// Read `MDArray` as one-dimensional string array
    pub fn read_as_string_array(&self) -> Result<Vec<String>> {
        let data_type = self.datatype();
        if data_type.class() != GDALExtendedDataTypeClass::GEDTC_STRING {
            // We have to check that the data type is string.
            // Only then, GDAL returns an array of string pointers.
            // Otherwise, we will dereference these string pointers and get a segfault.

            return Err(GdalError::UnsupportedMdDataType {
                data_type,
                method_name: "GDALMDArrayRead (string)",
            });
        }

        let num_values = self.num_elements() as usize;
        let mut string_pointers: Vec<*const c_char> = vec![std::ptr::null(); num_values];

        let count: Vec<usize> = self
            .dimensions()?
            .into_iter()
            .map(|dim| dim.size())
            .collect();
        let array_start_index: Vec<u64> = vec![0; count.len()];

        // If set to nullptr, [1, 1, … 1] will be used as a default to indicate consecutive elements.
        let array_step: *const i64 = std::ptr::null();
        // If set to nullptr, will be set so that pDstBuffer is written in a compact way,
        // with elements of the last / fastest varying dimension being consecutive.
        let buffer_stride: *const i64 = std::ptr::null();

        let p_dst_buffer_alloc_start: *mut libc::c_void = std::ptr::null_mut();
        let n_dst_buffer_alloc_size = 0;

        unsafe {
            let data_type = GDALMDArrayGetDataType(self.c_mdarray);

            let rv = gdal_sys::GDALMDArrayRead(
                self.c_mdarray,
                array_start_index.as_ptr(),
                count.as_ptr(),
                array_step,
                buffer_stride,
                data_type,
                string_pointers.as_mut_ptr().cast::<std::ffi::c_void>(),
                p_dst_buffer_alloc_start,
                n_dst_buffer_alloc_size,
            );

            GDALExtendedDataTypeRelease(data_type);

            // `rv` is boolean
            if rv != 1 {
                // OSGeo Python wrapper treats it as `CE_Failure`
                return Err(_last_cpl_err(CPLErr::CE_Failure));
            }

            let strings = string_pointers
                .into_iter()
                .map(|string_ptr| {
                    let string = _string(string_ptr);

                    VSIFree(string_ptr as *mut c_void);

                    string
                })
                .collect();

            Ok(strings)
        }
    }

    pub fn spatial_reference(&self) -> Result<SpatialRef> {
        unsafe {
            let c_gdal_spatial_ref = GDALMDArrayGetSpatialRef(self.c_mdarray);

            let gdal_spatial_ref = SpatialRef::from_c_obj(c_gdal_spatial_ref);

            OSRDestroySpatialReference(c_gdal_spatial_ref);

            gdal_spatial_ref
        }
    }

    pub fn no_data_value_as_double(&self) -> Option<f64> {
        let mut has_nodata = 0;

        let no_data_value = unsafe {
            GDALMDArrayGetNoDataValueAsDouble(self.c_mdarray, std::ptr::addr_of_mut!(has_nodata))
        };

        if has_nodata == 0 {
            None
        } else {
            Some(no_data_value)
        }
    }

    pub fn unit(&self) -> String {
        unsafe {
            // should not be freed
            let c_unit = GDALMDArrayGetUnit(self.c_mdarray);

            _string(c_unit)
        }
    }

    pub fn attribute(&self, name: &str) -> Result<Attribute> {
        let name = CString::new(name)?;

        unsafe {
            let c_attribute = GDALMDArrayGetAttribute(self.c_mdarray, name.as_ptr());

            if c_attribute.is_null() {
                return Err(_last_null_pointer_err("GDALGroupGetAttribute"));
            }

            Ok(Attribute::from_c_attribute(c_attribute))
        }
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
    _dataset: &'a Dataset,
}

impl Drop for Group<'_> {
    fn drop(&mut self) {
        unsafe {
            GDALGroupRelease(self.c_group);
        }
    }
}

impl<'a> Group<'a> {
    /// Create a Group from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_group(_dataset: &'a Dataset, c_group: GDALGroupH) -> Self {
        Group { c_group, _dataset }
    }

    pub fn name(&self) -> String {
        _string(unsafe { GDALGroupGetName(self.c_group) })
    }

    pub fn group_names(&self, options: CslStringList) -> Vec<String> {
        unsafe {
            let c_group_names = GDALGroupGetGroupNames(self.c_group, options.as_ptr());

            let strings = _string_array(c_group_names);

            CSLDestroy(c_group_names);

            strings
        }
    }

    pub fn array_names(&self, options: CslStringList) -> Vec<String> {
        unsafe {
            let c_array_names = GDALGroupGetMDArrayNames(self.c_group, options.as_ptr());

            let strings = _string_array(c_array_names);

            CSLDestroy(c_array_names);

            strings
        }
    }

    pub fn open_md_array(&self, name: &str, options: CslStringList) -> Result<MDArray> {
        let name = CString::new(name)?;

        unsafe {
            let c_mdarray = GDALGroupOpenMDArray(self.c_group, name.as_ptr(), options.as_ptr());

            if c_mdarray.is_null() {
                return Err(_last_null_pointer_err("GDALGroupOpenMDArray"));
            }

            Ok(MDArray::from_c_mdarray_and_group(self, c_mdarray))
        }
    }

    pub fn open_group(&'_ self, name: &str, options: CslStringList) -> Result<Group<'a>> {
        let name = CString::new(name)?;

        unsafe {
            let c_group = GDALGroupOpenGroup(self.c_group, name.as_ptr(), options.as_ptr());

            if c_group.is_null() {
                return Err(_last_null_pointer_err("GDALGroupOpenGroup"));
            }

            Ok(Group::from_c_group(self._dataset, c_group))
        }
    }

    pub fn attribute(&self, name: &str) -> Result<Attribute> {
        let name = CString::new(name)?;

        unsafe {
            let c_attribute = GDALGroupGetAttribute(self.c_group, name.as_ptr());

            if c_attribute.is_null() {
                return Err(_last_null_pointer_err("GDALGroupGetAttribute"));
            }

            Ok(Attribute::from_c_attribute(c_attribute))
        }
    }

    pub fn dimensions(&self, options: CslStringList) -> Result<Vec<Dimension>> {
        unsafe {
            let mut num_dimensions: usize = 0;
            let c_dimensions = GDALGroupGetDimensions(
                self.c_group,
                std::ptr::addr_of_mut!(num_dimensions),
                options.as_ptr(),
            );

            // `num_dimensions` is `0`, we can safely return an empty vector
            // `GDALGroupGetDimensions` does not state that errors can occur
            if num_dimensions > 0 && c_dimensions.is_null() {
                return Err(_last_null_pointer_err("GDALGroupGetDimensions"));
            }

            let dimensions_ref = std::slice::from_raw_parts_mut(c_dimensions, num_dimensions);

            let mut dimensions: Vec<Dimension> = Vec::with_capacity(num_dimensions);

            for c_dimension in dimensions_ref {
                let dimension =
                    Dimension::from_c_dimension(GroupOrArray::Group { _group: self }, *c_dimension);
                dimensions.push(dimension);
            }

            // only free the array, not the dimensions themselves
            VSIFree(c_dimensions as *mut c_void);

            Ok(dimensions)
        }
    }
}

/// A `GDALDimension` with name and size
#[derive(Debug)]
pub struct Dimension<'a> {
    c_dimension: *mut GDALDimensionHS,
    _parent: GroupOrArray<'a>,
}

impl Drop for Dimension<'_> {
    fn drop(&mut self) {
        unsafe {
            GDALDimensionRelease(self.c_dimension);
        }
    }
}

impl<'a> Dimension<'a> {
    /// Create a MDArray from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_dimension(
        _parent: GroupOrArray<'a>,
        c_dimension: *mut GDALDimensionHS,
    ) -> Self {
        Self {
            c_dimension,
            _parent,
        }
    }
    pub fn size(&self) -> usize {
        unsafe { GDALDimensionGetSize(self.c_dimension) as usize }
    }

    pub fn name(&self) -> String {
        _string(unsafe { GDALDimensionGetName(self.c_dimension) })
    }

    pub fn indexing_variable(&self) -> MDArray {
        unsafe {
            let c_md_array = GDALDimensionGetIndexingVariable(self.c_dimension);

            MDArray::from_c_mdarray_and_dimension(self, c_md_array)
        }
    }
}

/// Wrapper for `GDALExtendedDataType`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendedDataType {
    c_data_type: GDALExtendedDataTypeH,
}

impl Drop for ExtendedDataType {
    fn drop(&mut self) {
        unsafe {
            GDALExtendedDataTypeRelease(self.c_data_type);
        }
    }
}

impl ExtendedDataType {
    /// Create an `ExtendedDataTypeNumeric` from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub fn from_c_extended_data_type(c_data_type: GDALExtendedDataTypeH) -> Self {
        Self { c_data_type }
    }

    /// The result is only valid if the data type is numeric
    pub fn class(&self) -> GDALExtendedDataTypeClass::Type {
        unsafe { GDALExtendedDataTypeGetClass(self.c_data_type) }
    }

    /// The result is only valid if the data type is numeric
    pub fn numeric_datatype(&self) -> GDALDataType::Type {
        unsafe { GDALExtendedDataTypeGetNumericDataType(self.c_data_type) }
    }
}

// Wrapper for `GDALAttribute`
#[derive(Debug)]
pub struct Attribute {
    c_attribute: GDALAttributeH,
}

impl Drop for Attribute {
    fn drop(&mut self) {
        unsafe {
            GDALAttributeRelease(self.c_attribute);
        }
    }
}

impl Attribute {
    /// Create an `ExtendedDataTypeNumeric` from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub fn from_c_attribute(c_attribute: GDALAttributeH) -> Self {
        Self { c_attribute }
    }

    /// Return the size of the dimensions of the attribute.
    /// This will be an empty array for a scalar (single value) attribute.
    pub fn dimension_sizes(&self) -> Vec<usize> {
        unsafe {
            let mut num_dimensions = 0;

            let c_dimension_sizes = GDALAttributeGetDimensionsSize(
                self.c_attribute,
                std::ptr::addr_of_mut!(num_dimensions),
            );

            let dimension_sizes = std::slice::from_raw_parts(c_dimension_sizes, num_dimensions)
                .iter()
                .map(|&size| size as usize)
                .collect();

            VSIFree(c_dimension_sizes as *mut c_void);

            dimension_sizes
        }
    }

    pub fn datatype(&self) -> ExtendedDataType {
        unsafe {
            let c_data_type = GDALAttributeGetDataType(self.c_attribute);
            ExtendedDataType::from_c_extended_data_type(c_data_type)
        }
    }

    pub fn read_as_string(&self) -> String {
        unsafe {
            // SAFETY: should no be freed
            let c_string = GDALAttributeReadAsString(self.c_attribute);

            _string(c_string)
        }
    }

    pub fn read_as_string_array(&self) -> Vec<String> {
        unsafe {
            let c_string_array = GDALAttributeReadAsStringArray(self.c_attribute);

            let string_array = _string_array(c_string_array);

            CSLDestroy(c_string_array);

            string_array
        }
    }

    pub fn read_as_i64(&self) -> i32 {
        unsafe { GDALAttributeReadAsInt(self.c_attribute) }
    }

    pub fn read_as_i64_array(&self) -> Vec<i32> {
        unsafe {
            let mut array_len = 0;
            let c_int_array =
                GDALAttributeReadAsIntArray(self.c_attribute, std::ptr::addr_of_mut!(array_len));

            let int_array = std::slice::from_raw_parts(c_int_array, array_len).to_vec();

            VSIFree(c_int_array as *mut c_void);

            int_array
        }
    }

    pub fn read_as_f64(&self) -> f64 {
        unsafe { GDALAttributeReadAsDouble(self.c_attribute) }
    }

    pub fn read_as_f64_array(&self) -> Vec<f64> {
        unsafe {
            let mut array_len = 0;
            let c_int_array =
                GDALAttributeReadAsDoubleArray(self.c_attribute, std::ptr::addr_of_mut!(array_len));

            let float_array = std::slice::from_raw_parts(c_int_array, array_len).to_vec();

            VSIFree(c_int_array as *mut c_void);

            float_array
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Dataset, DatasetOptions, GdalOpenFlags};

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
        let options = CslStringList::new(); //Driver specific options determining how groups should be retrieved. Pass nullptr for default behavior.
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
        let options = CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.open_md_array(&array_name, options).unwrap();
        let n_dimension = md_array.num_dimensions();
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
        let options = CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.open_md_array(&array_name, options).unwrap();
        let n_elements = md_array.num_elements();
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

        // group dimensions
        let group_dimensions = root_group.dimensions(CslStringList::new()).unwrap();
        let group_dimensions_names: Vec<String> = group_dimensions
            .into_iter()
            .map(|dimensions| dimensions.name())
            .collect();
        assert_eq!(group_dimensions_names, vec!["x", "y"]);

        // array dimensions

        let array_name = "Band1".to_string();
        let options = CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.open_md_array(&array_name, options).unwrap();
        let dimensions = md_array.dimensions().unwrap();
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
        let options = CslStringList::new(); //Driver specific options determining how the array should be opened. Pass nullptr for default behavior.
        let md_array = root_group.open_md_array(&array_name, options).unwrap();
        let dimensions = md_array.dimensions().unwrap();
        let mut dimensions_size = Vec::new();
        for dimension in dimensions {
            dimensions_size.push(dimension.size());
        }
        assert_eq!(dimensions_size, vec![20, 20])
    }

    #[test]
    fn test_read_data() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();

        let root_group = dataset.root_group().unwrap();
        let md_array = root_group
            .open_md_array("Band1", CslStringList::new())
            .unwrap();

        let values = md_array.read_as::<u8>(vec![0, 0], vec![20, 20]).unwrap();

        assert_eq!(&values[..4], &[181, 181, 156, 148]);
    }

    #[test]
    fn test_read_string_array() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/alldatatypes.nc", dataset_options).unwrap();

        let root_group = dataset.root_group().unwrap();

        let string_array = root_group
            .open_md_array("string_var", CslStringList::new())
            .unwrap();

        assert_eq!(string_array.read_as_string_array().unwrap(), ["abcd", "ef"]);

        let non_string_array = root_group
            .open_md_array("uint_var", CslStringList::new())
            .unwrap();

        // check that we don't get a `SIGSEV` here
        assert!(non_string_array.read_as_string_array().is_err());
    }

    #[test]
    fn test_datatype() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();

        let root_group = dataset.root_group().unwrap();

        let md_array = root_group
            .open_md_array("Band1", CslStringList::new())
            .unwrap();

        let datatype = md_array.datatype();

        assert_eq!(datatype.class(), GDALExtendedDataTypeClass::GEDTC_NUMERIC);
        assert_eq!(datatype.numeric_datatype(), GDALDataType::GDT_Byte);
    }

    #[test]
    fn test_spatial_ref() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();

        let root_group = dataset.root_group().unwrap();
        let md_array = root_group
            .open_md_array("Band1", CslStringList::new())
            .unwrap();

        let spatial_ref = md_array.spatial_reference().unwrap();

        assert_eq!(spatial_ref.name().unwrap(), "NAD27 / UTM zone 11N");

        assert_eq!(spatial_ref.authority().unwrap(), "EPSG:26711");
    }

    #[test]
    fn test_no_data_value() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();

        let root_group = dataset.root_group().unwrap();
        let md_array = root_group
            .open_md_array("Band1", CslStringList::new())
            .unwrap();

        assert_eq!(md_array.no_data_value_as_double(), Some(0.));
    }

    #[test]
    fn test_attributes() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/cf_nasa_4326.nc", dataset_options).unwrap();

        let root_group = dataset.root_group().unwrap();

        assert_eq!(
            root_group.attribute("title").unwrap().read_as_string(),
            "Simple CF file"
        );

        let group_science = root_group
            .open_group("science", CslStringList::new())
            .unwrap();

        assert!(group_science
            .dimensions(Default::default())
            .unwrap()
            .is_empty());

        let group_grids = group_science
            .open_group("grids", CslStringList::new())
            .unwrap();
        let group_data = group_grids
            .open_group("data", CslStringList::new())
            .unwrap();

        let md_array = group_data
            .open_md_array("temp", CslStringList::new())
            .unwrap();

        assert_eq!(
            md_array
                .attribute("standard_name")
                .unwrap()
                .read_as_string(),
            "air_temperature"
        );

        assert_eq!(
            md_array.attribute("_FillValue").unwrap().read_as_f64(),
            -9999.
        );
    }

    #[test]
    fn test_unit() {
        let dataset_options = DatasetOptions {
            open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
            allowed_drivers: None,
            open_options: None,
            sibling_files: None,
        };
        let dataset = Dataset::open_ex("fixtures/cf_nasa_4326.nc", dataset_options).unwrap();

        let root_group = dataset.root_group().unwrap();

        assert_eq!(
            root_group.attribute("title").unwrap().read_as_string(),
            "Simple CF file"
        );

        let group_science = root_group
            .open_group("science", CslStringList::new())
            .unwrap();
        let group_grids = group_science
            .open_group("grids", CslStringList::new())
            .unwrap();

        drop(group_science); // check that `Group`s do not borrow each other

        let group_data = group_grids
            .open_group("data", CslStringList::new())
            .unwrap();

        let md_array = group_data
            .open_md_array("temp", CslStringList::new())
            .unwrap();

        assert_eq!(md_array.unit(), "K");
    }
}
