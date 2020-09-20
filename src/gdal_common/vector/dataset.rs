use crate::utils::_last_null_pointer_err;
use gdal_sys::{self, OGRLayerH, OGRwkbGeometryType};
use libc::c_int;
use std::ffi::CString;
use std::ptr::null_mut;

use crate::{errors::*, Dataset, DatasetCommon, Layer, SpatialRef, SpatialRefCommon};

pub trait VectorDatasetCommon: DatasetCommon {
    /// Get number of layers.
    fn count_vector_layers(&self) -> isize {
        (unsafe { gdal_sys::OGR_DS_GetLayerCount(self.c_dataset()) }) as isize
    }

    fn _child_layer(&self, c_layer: OGRLayerH) -> Layer {
        let layer = unsafe { Layer::from_c_layer(c_layer, self.as_ref()) };
        layer
    }

    /// Get layer number `idx`.
    fn layer(&mut self, idx: isize) -> Result<Layer> {
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.c_dataset(), idx as c_int) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayer").into());
        }
        Ok(self._child_layer(c_layer))
    }

    /// Get layer with `name`.
    fn layer_by_name(&mut self, name: &str) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayerByName(self.c_dataset(), c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayerByName").into());
        }
        Ok(self._child_layer(c_layer))
    }

    /// Create a new layer with a blank definition.
    fn create_layer(&mut self) -> Result<Layer> {
        let c_name = CString::new("")?;
        let c_layer = unsafe {
            gdal_sys::OGR_DS_CreateLayer(
                self.c_dataset(),
                c_name.as_ptr(),
                null_mut(),
                OGRwkbGeometryType::wkbUnknown,
                null_mut(),
            )
        };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_CreateLayer").into());
        };
        Ok(self._child_layer(c_layer))
    }

    /// Create a new layer with name, spatial ref. and type.
    fn create_layer_ext(
        &mut self,
        name: &str,
        srs: Option<&SpatialRef>,
        ty: OGRwkbGeometryType::Type,
    ) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_srs = match srs {
            Some(srs) => srs.c_spatial_ref(),
            None => null_mut(),
        };

        let c_layer = unsafe {
            gdal_sys::OGR_DS_CreateLayer(self.c_dataset(), c_name.as_ptr(), c_srs, ty, null_mut())
        };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_CreateLayer").into());
        };
        Ok(self._child_layer(c_layer))
    }
}

impl VectorDatasetCommon for Dataset {}
