use std::{ffi::CString, path::Path, ptr, sync::Once};

use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string};
use crate::{
    gdal_major_object::MajorObject, raster::RasterBand, spatial_ref::SpatialRef, vector::Layer,
    Driver, Metadata,
};
use gdal_sys::{
    self, CPLErr, GDALAccess, GDALDatasetH, GDALMajorObjectH, OGRLayerH, OGRwkbGeometryType,
};
use libc::{c_double, c_int};
use ptr::null_mut;

use crate::errors::*;

pub type GeoTransform = [c_double; 6];
static START: Once = Once::new();

pub struct Dataset {
    c_dataset: GDALDatasetH,
}

pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            gdal_sys::GDALAllRegister();
        });
    }
}

impl Dataset {
    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_dataset(&self) -> GDALDatasetH {
        self.c_dataset
    }

    pub fn open(path: &Path) -> Result<Dataset> {
        Self::open_ex(path, None, None, None, None)
    }

    pub fn open_ex(
        path: &Path,
        open_flags: Option<u32>,
        _allowed_drivers: Option<&str>, // TODO: use parameters
        _open_options: Option<&str>,
        _sibling_files: Option<&str>,
    ) -> Result<Dataset> {
        _register_drivers();
        let filename = path.to_string_lossy();
        let c_filename = CString::new(filename.as_ref())?;
        let c_open_flags = open_flags.unwrap_or(GDALAccess::GA_ReadOnly); // This defaults to GdalAccess::GA_ReadOnly

        let c_dataset = unsafe {
            gdal_sys::GDALOpenEx(
                c_filename.as_ptr(),
                c_open_flags,
                ptr::null(),
                ptr::null(),
                ptr::null(),
            )
        };
        if c_dataset.is_null() {
            return Err(_last_null_pointer_err("GDALOpenEx").into());
        }
        Ok(Dataset { c_dataset })
    }

    /// Creates a new Dataset by wrapping a C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_dataset(c_dataset: GDALDatasetH) -> Dataset {
        Dataset { c_dataset }
    }

    pub fn projection(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetProjectionRef(self.c_dataset) };
        _string(rv)
    }

    pub fn set_projection(&self, projection: &str) -> Result<()> {
        let c_projection = CString::new(projection)?;
        unsafe { gdal_sys::GDALSetProjection(self.c_dataset, c_projection.as_ptr()) };
        Ok(())
    }

    pub fn create_copy(&self, driver: &Driver, filename: &str) -> Result<Dataset> {
        let c_filename = CString::new(filename)?;
        let c_dataset = unsafe {
            gdal_sys::GDALCreateCopy(
                driver.c_driver(),
                c_filename.as_ptr(),
                self.c_dataset,
                0,
                ptr::null_mut(),
                None,
                ptr::null_mut(),
            )
        };
        if c_dataset.is_null() {
            return Err(_last_null_pointer_err("GDALCreateCopy").into());
        }
        Ok(unsafe { Dataset::from_c_dataset(c_dataset) })
    }

    pub fn driver(&self) -> Driver {
        unsafe {
            let c_driver = gdal_sys::GDALGetDatasetDriver(self.c_dataset);
            Driver::from_c_driver(c_driver)
        }
    }

    pub fn rasterband(&self, band_index: isize) -> Result<RasterBand> {
        unsafe {
            let c_band = gdal_sys::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            if c_band.is_null() {
                return Err(_last_null_pointer_err("GDALGetRasterBand").into());
            }
            Ok(RasterBand::_with_c_ptr(c_band, self))
        }
    }

    fn child_layer(&self, c_layer: OGRLayerH) -> Layer {
        unsafe { Layer::from_c_layer(c_layer, self) }
    }

    pub fn layer_count(&self) -> isize {
        (unsafe { gdal_sys::OGR_DS_GetLayerCount(self.c_dataset) }) as isize
    }

    pub fn layer(&mut self, idx: isize) -> Result<Layer> {
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayer").into());
        }
        Ok(self.child_layer(c_layer))
    }

    pub fn layer_by_name(&mut self, name: &str) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_layer = unsafe { gdal_sys::OGR_DS_GetLayerByName(self.c_dataset(), c_name.as_ptr()) };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_GetLayerByName").into());
        }
        Ok(self.child_layer(c_layer))
    }

    pub fn raster_count(&self) -> isize {
        (unsafe { gdal_sys::GDALGetRasterCount(self.c_dataset) }) as isize
    }

    pub fn raster_size(&self) -> (usize, usize) {
        let size_x = unsafe { gdal_sys::GDALGetRasterXSize(self.c_dataset) } as usize;
        let size_y = unsafe { gdal_sys::GDALGetRasterYSize(self.c_dataset) } as usize;
        (size_x, size_y)
    }

    /// Create a new layer with a blank name, no `SpatialRef`, and without (wkbUnknown) geometry type.
    pub fn create_layer_blank(&mut self) -> Result<Layer> {
        self.create_layer("", None, OGRwkbGeometryType::wkbUnknown)
    }

    /// Create a new layer with a name, an optional `SpatialRef`, and a geometry type.
    pub fn create_layer(
        &mut self,
        name: &str,
        srs: Option<&SpatialRef>,
        ty: OGRwkbGeometryType::Type,
    ) -> Result<Layer> {
        let c_name = CString::new(name)?;
        let c_srs = match srs {
            Some(srs) => srs.to_c_hsrs(),
            None => null_mut(),
        };

        let c_layer = unsafe {
            gdal_sys::OGR_DS_CreateLayer(self.c_dataset, c_name.as_ptr(), c_srs, ty, null_mut())
        };
        if c_layer.is_null() {
            return Err(_last_null_pointer_err("OGR_DS_CreateLayer").into());
        };
        Ok(self.child_layer(c_layer))
    }

    /// Affine transformation called geotransformation.
    ///
    /// This is like a linear transformation preserves points, straight lines and planes.
    /// Also, sets of parallel lines remain parallel after an affine transformation.
    /// # Arguments
    /// * transformation - coeficients of transformations
    ///
    /// x-coordinate of the top-left corner pixel (x-offset)
    /// width of a pixel (x-resolution)
    /// row rotation (typically zero)
    /// y-coordinate of the top-left corner pixel
    /// column rotation (typically zero)
    /// height of a pixel (y-resolution, typically negative)
    pub fn set_geo_transform(&self, transformation: &GeoTransform) -> Result<()> {
        assert_eq!(transformation.len(), 6);
        let rv = unsafe {
            gdal_sys::GDALSetGeoTransform(self.c_dataset, transformation.as_ptr() as *mut f64)
        };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv).into());
        }
        Ok(())
    }

    /// Get affine transformation coefficients.
    ///
    /// x-coordinate of the top-left corner pixel (x-offset)
    /// width of a pixel (x-resolution)
    /// row rotation (typically zero)
    /// y-coordinate of the top-left corner pixel
    /// column rotation (typically zero)
    /// height of a pixel (y-resolution, typically negative)
    pub fn geo_transform(&self) -> Result<GeoTransform> {
        let mut transformation = GeoTransform::default();
        let rv =
            unsafe { gdal_sys::GDALGetGeoTransform(self.c_dataset, transformation.as_mut_ptr()) };

        // check if the dataset has a GeoTransform
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv).into());
        }
        Ok(transformation)
    }
}

impl MajorObject for Dataset {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_dataset
    }
}

impl Metadata for Dataset {}

impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::GDALClose(self.c_dataset);
        }
    }
}
