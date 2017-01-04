use libc::{c_int, c_double, c_void};
use std::ffi::{CString};
use std::path::Path;
use utils::{_string, _last_cpl_err};
use raster::{Driver, RasterBand};
use raster::driver::_register_drivers;
use raster::gdal_enums::{GDALAccess, GDALDataType};
use raster::types::GdalType;
use gdal_major_object::MajorObject;
use metadata::Metadata;
use gdal_sys::{gdal, cpl_error};

use errors::*;

pub type GeoTransform = [c_double; 6];

pub struct Dataset {
    c_dataset: *const c_void,
}

impl MajorObject for Dataset {
    unsafe fn gdal_object_ptr(&self) -> *const c_void {
        self.c_dataset
    }
}

impl Metadata for Dataset {}

impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe { gdal::GDALClose(self.c_dataset); }
    }
}


impl Dataset {
    pub fn open(path: &Path) -> Result<Dataset> {
        _register_drivers();
        let filename = path.to_str().unwrap();
        let c_filename = CString::new(filename.as_bytes()).unwrap();
        let c_dataset = unsafe { gdal::GDALOpen(c_filename.as_ptr(), GDALAccess::GA_ReadOnly) };
        if c_dataset.is_null() {
            return Err(ErrorKind::NullPointer("GDALOpen").into());
        }
        Ok(Dataset{c_dataset: c_dataset})
    }

    pub unsafe fn _with_c_ptr(c_dataset: *const c_void) -> Dataset {
        return Dataset{c_dataset: c_dataset};
    }

    pub unsafe fn _c_ptr(&self) -> *const c_void {
        return self.c_dataset;
    }


    pub fn rasterband<'a>(&'a self, band_index: isize) -> Result<RasterBand<'a>> {
        unsafe {
            let c_band = gdal::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            if c_band.is_null() {
                return Err(ErrorKind::NullPointer("GDALGetRasterBand").into());
            }
            Ok(RasterBand::_with_c_ptr(c_band, self))
        }
    }

    pub fn size(&self) -> (usize, usize) {
        let size_x = unsafe { gdal::GDALGetRasterXSize(self.c_dataset) } as usize;
        let size_y = unsafe { gdal::GDALGetRasterYSize(self.c_dataset) } as usize;
        return (size_x, size_y);
    }

    pub fn driver(&self) -> Driver {
        unsafe {
            let c_driver = gdal::GDALGetDatasetDriver(self.c_dataset);
            return Driver::_with_c_ptr(c_driver);
        };
    }

    pub fn count(&self) -> isize {
        return unsafe { gdal::GDALGetRasterCount(self.c_dataset) } as isize;
    }

    pub fn projection(&self) -> String {
        let rv = unsafe { gdal::GDALGetProjectionRef(self.c_dataset) };
        return _string(rv);
    }

    pub fn set_projection(&self, projection: &str) {
        let c_projection = CString::new(projection.as_bytes()).unwrap();
        unsafe { gdal::GDALSetProjection(self.c_dataset, c_projection.as_ptr()) };
    }

    pub fn set_geo_transform(&self, tr: &GeoTransform) -> Result<()> {
        assert_eq!(tr.len(), 6);
        let rv = unsafe {
            gdal::GDALSetGeoTransform(self.c_dataset, tr.as_ptr())
        };
        if rv != cpl_error::CPLErr::CE_None {            
            return Err(_last_cpl_err(rv).into());
        }
        Ok(())
    }

    pub fn geo_transform(&self) -> Result<GeoTransform> {
        let mut tr = GeoTransform::default();
        let rv = unsafe {
            gdal::GDALGetGeoTransform(
                self.c_dataset,
                tr.as_mut_ptr()
            )
        };

        // check if the dataset has a GeoTransform
        if rv != cpl_error::CPLErr::CE_None {            
            return Err(_last_cpl_err(rv).into());
        }
        Ok(tr)
    }

    pub fn create_copy(
        &self,
        driver: Driver,
        filename: &str
    ) -> Result<Dataset> {
        use std::ptr::null;
        let c_filename = CString::new(filename.as_bytes()).unwrap();
        let c_dataset = unsafe { gdal::GDALCreateCopy(
                driver._c_ptr(),
                c_filename.as_ptr(),
                self.c_dataset,
                0,
                null(),
                null(),
                null()
            ) };
        if c_dataset.is_null() {
            return Err(ErrorKind::NullPointer("GDALCreateCopy").into());
        }
        Ok(Dataset{c_dataset: c_dataset})
    }

    pub fn band_type(&self, band_index: isize) -> Result<GDALDataType> {
        self.rasterband(band_index).map(|band| band.band_type())
    }

    /// Read a 'Buffer<u8>' from a 'Dataset'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * buffer_size - the desired size of the 'Buffer'
    pub fn read_raster(&self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize)
    ) -> Result<ByteBuffer>
    {
        self.read_raster_as::<u8>(
            band_index,
            window,
            window_size,
            size
        )
    }

    /// Read a full 'Dataset' as 'Buffer<T>'.
    /// # Arguments
    /// * band_index - the band_index
    pub fn read_full_raster_as<T: Copy + GdalType>(
        &self,
        band_index: isize,
    ) -> Result<Buffer<T>>
    {
        self.rasterband(band_index)?.read_band_as()
    }

    /// Read a 'Buffer<T>' from a 'Dataset'. T implements 'GdalType'
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * buffer_size - the desired size of the 'Buffer'
    pub fn read_raster_as<T: Copy + GdalType>(
        &self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
    ) -> Result<Buffer<T>>
    {
        self.rasterband(band_index)?.read_as(window, window_size, size)
    }

    /// Write a 'Buffer<T>' into a 'Dataset'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != Buffer.size)
    pub fn write_raster<T: GdalType+Copy>(
        &self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        buffer: Buffer<T>
    ) -> Result<()> {
        self.rasterband(band_index)?.write(window, window_size, buffer)
    }

}

pub struct Buffer<T: GdalType> {
    pub size: (usize, usize),
    pub data: Vec<T>,
}

impl<T: GdalType> Buffer<T> {
    pub fn new(size: (usize, usize), data: Vec<T>) -> Buffer<T> {
        Buffer{size: size, data: data}
    }
}

pub type ByteBuffer = Buffer<u8>;
