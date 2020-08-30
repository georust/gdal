use crate::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::spatial_ref::SpatialRef;
use crate::utils::_last_null_pointer_err;
use crate::vector::driver::_register_drivers;
use crate::vector::Layer;
use gdal_sys::{self, GDALMajorObjectH, OGRDataSourceH, OGRLayerH, OGRwkbGeometryType};
use libc::c_int;
use std::ffi::CString;
use std::path::Path;
use std::ptr::null_mut;

use crate::errors::*;

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
    c_dataset: OGRDataSourceH,
    layers: Vec<Layer>,
}

impl MajorObject for Dataset {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_dataset
    }
}

impl Metadata for Dataset {}

impl Dataset {
    pub unsafe fn _with_c_dataset(c_dataset: OGRDataSourceH) -> Dataset {
        Dataset {
            c_dataset,
            layers: vec![],
        }
    }

    /// Open the dataset at `path`.
    pub fn open(path: &Path) -> Result<Dataset> {
        _register_drivers();
        let filename = path.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_dataset = unsafe { gdal_sys::OGROpen(c_filename.as_ptr(), 0, null_mut()) };
        if c_dataset.is_null() {
            Err(_last_null_pointer_err("OGROpen"))?;
        };
        Ok(Dataset {
            c_dataset,
            layers: vec![],
        })
    }

    /// Get number of layers.
    pub fn count(&self) -> isize {
        (unsafe { gdal_sys::OGR_DS_GetLayerCount(self.c_dataset) }) as isize
    }

    fn _child_layer(&mut self, c_layer: OGRLayerH) -> &Layer {
        let layer = unsafe { Layer::_with_c_layer(c_layer) };
        self.layers.push(layer);
        self.layers.last().unwrap()
    }

    /// Get layer number `idx`.
    pub fn layer(&mut self, idx: isize) -> Result<&Layer> {
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        if c_layer.is_null() {
            Err(_last_null_pointer_err("OGR_DS_GetLayer"))?;
        }
        Ok(self._child_layer(c_layer))
    }

    /// Get layer with `name`.
    pub fn layer_by_name(&mut self, name: &str) -> Result<&Layer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayerByName(self.c_dataset, c_name.as_ptr()) };
        if c_layer.is_null() {
            Err(_last_null_pointer_err("OGR_DS_GetLayerByName"))?;
        }
        Ok(self._child_layer(c_layer))
    }

    /// Create a new layer with a blank definition.
    pub fn create_layer(&mut self) -> Result<&mut Layer> {
        let c_name = CString::new("")?;
        let c_layer = unsafe {
            gdal_sys::OGR_DS_CreateLayer(
                self.c_dataset,
                c_name.as_ptr(),
                null_mut(),
                OGRwkbGeometryType::wkbUnknown,
                null_mut(),
            )
        };
        if c_layer.is_null() {
            Err(_last_null_pointer_err("OGR_DS_CreateLayer"))?;
        };
        self._child_layer(c_layer);
        Ok(self.layers.last_mut().unwrap()) // TODO: is this safe?
    }

    /// Create a new layer with name, spatial ref. and type.
    pub fn create_layer_ext(
        &mut self,
        name: &str,
        srs: Option<&SpatialRef>,
        ty: OGRwkbGeometryType::Type,
    ) -> Result<&mut Layer> {
        let c_name = CString::new(name)?;
        let c_srs = match srs {
            Some(srs) => srs.to_c_hsrs(),
            None => null_mut(),
        };

        let c_layer = unsafe {
            gdal_sys::OGR_DS_CreateLayer(self.c_dataset, c_name.as_ptr(), c_srs, ty, null_mut())
        };
        if c_layer.is_null() {
            Err(_last_null_pointer_err("OGR_DS_CreateLayer"))?;
        };
        self._child_layer(c_layer);
        Ok(self.layers.last_mut().unwrap()) // TODO: is this safe?
    }

    /// Copy layers to another dataset.
    pub fn copy_layer(
        &mut self,
        layer: &Layer,
        name: &str
    ) -> Result<&mut Layer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe{
            gdal_sys::OGR_DS_CopyLayer(
                self.c_dataset,
                layer.c_layer(),
                c_name.as_ptr(),
                null_mut(),
            )
        };
        if c_layer.is_null() {
            Err(_last_null_pointer_err("OGR_DS_CopyLayer"))?;
        };
        self._child_layer(c_layer);
        Ok(self.layers.last_mut().unwrap()) // TODO: is this safe?
    }
}

impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::OGR_DS_Destroy(self.c_dataset);
        }
    }
}
