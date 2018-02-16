use std::ffi::CString;
use std::path::Path;
use std::ptr::null_mut;
use libc::{c_int, c_void};
use vector::{Layer};
use vector::driver::_register_drivers;
use gdal_major_object::MajorObject;
use metadata::Metadata;
use gdal_sys::{self, OGRwkbGeometryType};
use utils::{_last_null_pointer_err};


use errors::*;

/// Vector dataset
///
/// ```
/// use std::path::Path;
/// use gdal::vector::Dataset;
///
/// let mut dataset = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
/// println!("Dataset has {} layers", dataset.count());
/// ```
pub struct Dataset {
    c_dataset: *mut c_void,
    layers: Vec<Layer>,
}

impl MajorObject for Dataset {
    unsafe fn gdal_object_ptr(&self) -> *mut c_void {
        self.c_dataset
    }
}

impl Metadata for Dataset {}


impl Dataset {
    pub unsafe fn _with_c_dataset(c_dataset: *mut c_void) -> Dataset {
        Dataset{c_dataset: c_dataset, layers: vec!()}
    }

    /// Open the dataset at `path`.
    pub fn open(path: &Path) -> Result<Dataset> {
        _register_drivers();
        let filename = path.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_dataset = unsafe { gdal_sys::OGROpen(c_filename.as_ptr(), 0, null_mut()) };
        if c_dataset.is_null() {
           return Err(_last_null_pointer_err("OGROpen").into());
        };
        Ok(Dataset{c_dataset: c_dataset, layers: vec!()})
    }

    /// Get number of layers.
    pub fn count(&self) -> isize {
        return unsafe { gdal_sys::OGR_DS_GetLayerCount(self.c_dataset) } as isize;
    }

    fn _child_layer(&mut self, c_layer: *mut c_void) -> &Layer {
        let layer = unsafe { Layer::_with_c_layer(c_layer) };
        self.layers.push(layer);
        return self.layers.last().unwrap();
    }

    /// Get layer number `idx`.
    pub fn layer(&mut self, idx: isize) -> Result<&Layer> {
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayer").into());
        }
        Ok(self._child_layer(c_layer))
    }

    /// Get layer with `name`.
    pub fn layer_by_name(&mut self, name: &str) -> Result<&Layer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayerByName(self.c_dataset, c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayerByName").into());
        }
        Ok(self._child_layer(c_layer))
    }

    /// Create a new layer with a blank definition.
    pub fn create_layer(&mut self) -> Result<&mut Layer> {
        let c_name = CString::new("")?;
        let c_layer = unsafe { gdal_sys::OGR_DS_CreateLayer(
            self.c_dataset,
            c_name.as_ptr(),
            null_mut(),
            OGRwkbGeometryType::wkbUnknown,
            null_mut(),
        ) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_CreateLayer").into());
        };
        self._child_layer(c_layer);
        Ok(self.layers.last_mut().unwrap()) // TODO: is this safe?
    }
}


impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe { gdal_sys::OGR_DS_Destroy(self.c_dataset); }
    }
}
