use crate::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::raster::types::GdalType;
use crate::raster::{Buffer, Dataset};
use crate::utils::_last_cpl_err;
use crate::raster::GDALDataType;
use gdal_sys::{self, CPLErr, GDALMajorObjectH, GDALRWFlag, GDALRasterBandH};
use libc::c_int;

#[cfg(feature = "ndarray")]
use ndarray::Array2;

use crate::errors::*;

pub struct RasterBand<'a> {
    c_rasterband: GDALRasterBandH,
    owning_dataset: &'a Dataset,
}

impl<'a> RasterBand<'a> {
    pub fn owning_dataset(&self) -> &'a Dataset {
        self.owning_dataset
    }

    pub unsafe fn _with_c_ptr(c_rasterband: GDALRasterBandH, owning_dataset: &'a Dataset) -> Self {
        RasterBand {
            c_rasterband,
            owning_dataset,
        }
    }

    /// Get block size from a 'Dataset'.
    pub fn block_size(&self) -> (usize, usize) {
        let mut size_x = 0;
        let mut size_y = 0;

        unsafe { gdal_sys::GDALGetBlockSize(self.c_rasterband, &mut size_x, &mut size_y) };
        (size_x as usize, size_y as usize)
    }

    /// Get x-size of the band
    pub fn x_size(&self) -> usize {
        let out;
        unsafe {
            out = gdal_sys::GDALGetRasterBandXSize(self.c_rasterband);
        }
        out as usize
    }

    /// Get y-size of the band
    pub fn y_size(&self) -> usize {
        let out;
        unsafe { out = gdal_sys::GDALGetRasterBandYSize(self.c_rasterband) }
        out as usize
    }

    /// Get dimensions of the band.
    /// Note that this may not be the same as `size` on the
    /// `owning_dataset` due to scale.
    pub fn size(&self) -> (usize, usize) {
        (self.x_size(), self.y_size())
    }

    /// Read a 'Buffer<T>' from a 'Dataset'. T implements 'GdalType'
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * buffer_size - the desired size of the 'Buffer'
    pub fn read_as<T: Copy + GdalType>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
    ) -> Result<Buffer<T>> {
        let pixels = (size.0 * size.1) as usize;
        let mut data: Vec<T> = Vec::with_capacity(pixels);
        //let no_data:
        let rv = unsafe {
            gdal_sys::GDALRasterIO(
                self.c_rasterband,
                GDALRWFlag::GF_Read,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                data.as_mut_ptr() as GDALRasterBandH,
                size.0 as c_int,
                size.1 as c_int,
                T::gdal_type(),
                0,
                0,
            )
        };
        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))?;
        }

        unsafe {
            data.set_len(pixels);
        };

        Ok(Buffer { size, data })
    }

    #[cfg(feature = "ndarray")]
    /// Read a 'Array2<T>' from a 'Dataset'. T implements 'GdalType'.
    /// # Arguments
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != array_size)
    /// * array_size - the desired size of the 'Array'
    /// # Docs
    /// The Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis).
    pub fn read_as_array<T: Copy + GdalType>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        array_size: (usize, usize),
    ) -> Result<Array2<T>> {
        let pixels = (array_size.0 * array_size.1) as usize;
        let mut data: Vec<T> = Vec::with_capacity(pixels);

        let values = unsafe {
            gdal_sys::GDALRasterIO(
                self.c_rasterband,
                GDALRWFlag::GF_Read,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                data.as_mut_ptr() as GDALRasterBandH,
                array_size.0 as c_int,
                array_size.1 as c_int,
                T::gdal_type(),
                0,
                0,
            )
        };
        if values != CPLErr::CE_None {
            Err(_last_cpl_err(values))?;
        }

        unsafe {
            data.set_len(pixels);
        };

        // Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis)
        Array2::from_shape_vec((array_size.1, array_size.0), data).map_err(Into::into)
    }

    /// Read a full 'Dataset' as 'Buffer<T>'.
    /// # Arguments
    /// * band_index - the band_index
    pub fn read_band_as<T: Copy + GdalType>(&self) -> Result<Buffer<T>> {
        let size = self.owning_dataset.size();
        self.read_as::<T>(
            (0, 0),
            (size.0 as usize, size.1 as usize),
            (size.0 as usize, size.1 as usize),
        )
    }

    #[cfg(feature = "ndarray")]
    /// Read a 'Array2<T>' from a 'Dataset' block. T implements 'GdalType'
    /// # Arguments
    /// * block_index - the block index
    /// # Docs
    /// The Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis).
    pub fn read_block<T: Copy + GdalType>(&self, block_index: (usize, usize)) -> Result<Array2<T>> {
        let size = self.block_size();
        let pixels = (size.0 * size.1) as usize;
        let mut data: Vec<T> = Vec::with_capacity(pixels);

        //let no_data:
        let rv = unsafe {
            gdal_sys::GDALReadBlock(
                self.c_rasterband,
                block_index.0 as c_int,
                block_index.1 as c_int,
                data.as_mut_ptr() as GDALRasterBandH,
            )
        };
        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))?;
        }

        unsafe {
            data.set_len(pixels);
        };

        Array2::from_shape_vec((size.1, size.0), data).map_err(Into::into)
    }

    // Write a 'Buffer<T>' into a 'Dataset'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != Buffer.size)
    pub fn write<T: GdalType + Copy>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        buffer: &Buffer<T>,
    ) -> Result<()> {
        assert_eq!(buffer.data.len(), buffer.size.0 * buffer.size.1);
        let rv = unsafe {
            gdal_sys::GDALRasterIO(
                self.c_rasterband,
                GDALRWFlag::GF_Write,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                buffer.data.as_ptr() as GDALRasterBandH,
                buffer.size.0 as c_int,
                buffer.size.1 as c_int,
                T::gdal_type(),
                0,
                0,
            )
        };
        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))?;
        }
        Ok(())
    }

    pub fn band_type(&self) -> GDALDataType::Type {
        unsafe { gdal_sys::GDALGetRasterDataType(self.c_rasterband) }
    }

    pub fn no_data_value(&self) -> Option<f64> {
        let mut pb_success = 1;
        let no_data =
            unsafe { gdal_sys::GDALGetRasterNoDataValue(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(no_data as f64);
        }
        None
    }

    pub fn set_no_data_value(&self, no_data: f64) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALSetRasterNoDataValue(self.c_rasterband, no_data) };
        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))?;
        }
        Ok(())
    }

    pub fn scale(&self) -> Option<f64> {
        let mut pb_success = 1;
        let scale = unsafe { gdal_sys::GDALGetRasterScale(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(scale as f64);
        }
        None
    }

    pub fn offset(&self) -> Option<f64> {
        let mut pb_success = 1;
        let offset = unsafe { gdal_sys::GDALGetRasterOffset(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(offset as f64);
        }
        None
    }

    /// Get actual block size (at the edges) when block size
    /// does not divide band size.
    #[cfg(all(major_ge_2, minor_ge_2))]
    pub fn actual_block_size(&self, offset: (isize, isize)) -> Result<(usize, usize)> {
        let mut block_size_x = 0;
        let mut block_size_y = 0;
        let rv = unsafe {
            gdal_sys::GDALGetActualBlockSize(
                self.c_rasterband,
                offset.0 as libc::c_int,
                offset.1 as libc::c_int,
                &mut block_size_x,
                &mut block_size_y,
            )
        };
        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))?;
        }
        Ok((block_size_x as usize, block_size_y as usize))
    }
}

impl<'a> MajorObject for RasterBand<'a> {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_rasterband
    }
}

impl<'a> Metadata for RasterBand<'a> {}
