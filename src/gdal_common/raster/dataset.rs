use crate::utils::{_last_cpl_err, _last_null_pointer_err};
use crate::{GdalType, RasterBand, RasterBandCommon, RasterBuffer};
use gdal_sys::{self, CPLErr, GDALDataType};
use libc::{c_double, c_int};

#[cfg(feature = "ndarray")]
use ndarray::Array2;

use crate::{errors::*, Dataset, DatasetCommon};

pub type GeoTransform = [c_double; 6];

pub trait RasterDatasetCommon: DatasetCommon {
    fn rasterband(&self, band_index: isize) -> Result<RasterBand> {
        unsafe {
            let c_band = gdal_sys::GDALGetRasterBand(self.c_dataset(), band_index as c_int);
            if c_band.is_null() {
                Err(_last_null_pointer_err("GDALGetRasterBand"))?;
            }
            Ok(RasterBand::from_c_ptr(c_band, self.as_ref()))
        }
    }

    fn size(&self) -> (usize, usize) {
        let size_x = unsafe { gdal_sys::GDALGetRasterXSize(self.c_dataset()) } as usize;
        let size_y = unsafe { gdal_sys::GDALGetRasterYSize(self.c_dataset()) } as usize;
        (size_x, size_y)
    }

    fn raster_count(&self) -> isize {
        (unsafe { gdal_sys::GDALGetRasterCount(self.c_dataset()) }) as isize
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
    fn set_geo_transform(&self, transformation: &GeoTransform) -> Result<()> {
        assert_eq!(transformation.len(), 6);
        let rv = unsafe {
            gdal_sys::GDALSetGeoTransform(self.c_dataset(), transformation.as_ptr() as *mut f64)
        };
        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))?;
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
    fn geo_transform(&self) -> Result<GeoTransform> {
        let mut transformation = GeoTransform::default();
        let rv =
            unsafe { gdal_sys::GDALGetGeoTransform(self.c_dataset(), transformation.as_mut_ptr()) };

        // check if the dataset has a GeoTransform
        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))?;
        }
        Ok(transformation)
    }

    fn band_type(&self, band_index: isize) -> Result<GDALDataType::Type> {
        self.rasterband(band_index).map(|band| band.band_type())
    }

    /// Read a full 'Dataset' as 'RasterBuffer<T>'.
    /// # Arguments
    /// * band_index - the band_index
    fn read_full_raster<T: Copy + GdalType>(&self, band_index: isize) -> Result<RasterBuffer<T>> {
        self.rasterband(band_index)?.read_band_as()
    }

    /// Read a 'RasterBuffer<T>' from a 'Dataset'. T implements 'GdalType'
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * buffer_size - the desired size of the 'RasterBuffer'
    fn read_raster<T: Copy + GdalType>(
        &self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
    ) -> Result<RasterBuffer<T>> {
        self.rasterband(band_index)?
            .read_as(window, window_size, size)
    }

    #[cfg(feature = "ndarray")]
    /// Read a 'Array2<T>' from a 'Dataset'. T implements 'GdalType'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != array_size)
    /// * array_size - the desired size of the 'Array'
    fn read_as_array<T: Copy + GdalType>(
        &self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        array_size: (usize, usize),
    ) -> Result<Array2<T>> {
        self.rasterband(band_index)?
            .read_as_array(window, window_size, array_size)
    }

    /// Write a 'RasterBuffer<T>' into a 'Dataset'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != RasterBuffer.size)
    fn write_raster<T: GdalType + Copy>(
        &self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        buffer: &RasterBuffer<T>,
    ) -> Result<()> {
        self.rasterband(band_index)?
            .write(window, window_size, buffer)
    }
}

impl RasterDatasetCommon for Dataset {}
