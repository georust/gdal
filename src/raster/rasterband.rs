use libc::{c_int, c_void};
use raster::{Dataset, Buffer};
use raster::types::{GdalType};
use raster::gdal_enums;
use gdal_major_object::MajorObject;
use metadata::Metadata;
use gdal_sys::{gdal, cpl_error};
use utils::{_last_cpl_err};
use ndarray::Array2;
use num::FromPrimitive;
use std::any::TypeId;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use super::gdal_enums::GDALDataType;
use errors::ErrorKind::ConversionError;
use std::io::Cursor;

use errors::*;

pub struct RasterBand<'a> {
    c_rasterband: *const c_void,
    owning_dataset: &'a Dataset,
}

impl <'a> RasterBand<'a> {
    pub fn owning_dataset(&self) -> &'a Dataset {
        self.owning_dataset
    }

    pub unsafe fn _with_c_ptr(c_rasterband: *const c_void, owning_dataset: &'a Dataset) -> Self {
        RasterBand { c_rasterband: c_rasterband, owning_dataset: owning_dataset }
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
    ) -> Result<Buffer<T>>
    {
        let pixels = (size.0 * size.1) as usize;
        let mut data: Vec<T> = Vec::with_capacity(pixels);
        //let no_data:
        let rv = unsafe {
            gdal::GDALRasterIO(
                self.c_rasterband,
                gdal_enums::GDALRWFlag::GF_Read,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                data.as_mut_ptr() as *const c_void,
                size.0 as c_int,
                size.1 as c_int,
                T::gdal_type(),
                0,
                0
            )
        };
        if rv != cpl_error::CPLErr::CE_None {            
            return Err(_last_cpl_err(rv).into());
        }
        
        unsafe {
            data.set_len(pixels);
        };

        Ok(Buffer{
            size: size,
            data: data,
        })
    }

    /// Read a full 'Dataset' as 'Buffer<T>'.
    /// # Arguments
    /// * band_index - the band_index
    pub fn read_band_as<T: Copy + GdalType>(
        &self,
    ) -> Result<Buffer<T>>
    {
        let size = self.owning_dataset.size();
        self.read_as::<T>(
            (0, 0),
            (size.0 as usize, size.1 as usize),
            (size.0 as usize, size.1 as usize)
        )
    }

    /// Read a full 'Dataset' as an 'ndarray::Array2<T>'.
    /// # Arguments
    /// * band_index - the band_index
    pub fn read_band_as_array<U: 'static + Copy + FromPrimitive>(&self) -> Result<Array2<U>> {
        read_to_array::<U>(&self)
    }

    // Write a 'Buffer<T>' into a 'Dataset'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != Buffer.size)
    pub fn write<T: GdalType+Copy>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        buffer: Buffer<T>
    ) -> Result<()> {
        assert_eq!(buffer.data.len(), buffer.size.0 * buffer.size.1);
        let rv = unsafe { gdal::GDALRasterIO(
            self.c_rasterband,
            gdal_enums::GDALRWFlag::GF_Write,
            window.0 as c_int,
            window.1 as c_int,
            window_size.0 as c_int,
            window_size.1 as c_int,
            buffer.data.as_ptr() as *const c_void,
            buffer.size.0 as c_int,
            buffer.size.1 as c_int,
            T::gdal_type(),
            0,
            0
            )};
        if rv != cpl_error::CPLErr::CE_None {            
            return Err(_last_cpl_err(rv).into());
        }
        Ok(())
    }

    pub fn band_type(&self) -> gdal_enums::GDALDataType {

        let gdal_type: c_int;
        unsafe{
            gdal_type = gdal::GDALGetRasterDataType(self.c_rasterband);
        }
        gdal_enums::GDALDataType::from_c_int(gdal_type)
    }

    pub fn no_data_value(&self) ->Option<f64> {
        unsafe {
            let mut pb_success: c_int = 1;
            let raw_pb_success = &mut pb_success as *mut c_int;
            let no_data = gdal::GDALGetRasterNoDataValue(self.c_rasterband, raw_pb_success);
            if pb_success == 1 {
                return Some(no_data as f64);
            }
        }
        None
    }

    pub fn scale(&self) ->Option<f64> {
        unsafe {
            let mut pb_success: c_int = 1;
            let raw_pb_success = &mut pb_success as *mut c_int;
            let scale = gdal::GDALGetRasterScale(self.c_rasterband, raw_pb_success);
            if pb_success == 1 {
                return Some(scale as f64);
            }
        }
        None
    }

    pub fn offset(&self) ->Option<f64> {
        unsafe {
            let mut pb_success: c_int = 1;
            let raw_pb_success = &mut pb_success as *mut c_int;
            let offset = gdal::GDALGetRasterOffset(self.c_rasterband, raw_pb_success);
            if pb_success == 1 {
                return Some(offset as f64);
            }
        }
        None
    }
}

impl<'a> MajorObject for RasterBand<'a> {
    unsafe fn gdal_object_ptr(&self) -> *const c_void {
        self.c_rasterband
    }
}

impl<'a> Metadata for RasterBand<'a> {}

/// # Read To Array
/// A helper function that takes a raster band as input. It will
/// read the rasterband into an ndarray.
fn read_to_array<T: 'static + Copy + FromPrimitive>(rband: &RasterBand) -> Result<Array2<T>> {
    // get the data type of the dataset
    let gt: GDALDataType = rband.band_type();
    // get the byte buffer of the full dataset
    let buffer: Buffer<u8> = rband.read_band_as::<u8>()?;
    // get the data from the byte buffer
    let shape: (usize, usize) = buffer.size;
    let data = buffer.data;
    // convert the bytes of the buffer to type T
    return match gt {
        GDALDataType::GDT_Byte => Ok(Array2::from_shape_vec(shape, extract::<T>(data)?)?),
        GDALDataType::GDT_UInt16 => Ok(Array2::from_shape_vec(shape, extract::<T>(data)?)?),
        GDALDataType::GDT_Int16 => Ok(Array2::from_shape_vec(shape, extract::<T>(data)?)?),
        GDALDataType::GDT_UInt32 => Ok(Array2::from_shape_vec(shape, extract::<T>(data)?)?),
        GDALDataType::GDT_Int32 => Ok(Array2::from_shape_vec(shape, extract::<T>(data)?)?),
        GDALDataType::GDT_Float32 => Ok(Array2::from_shape_vec(shape, extract::<T>(data)?)?),
        GDALDataType::GDT_Float64 => Ok(Array2::from_shape_vec(shape, extract::<T>(data)?)?),
        _ => Err(ConversionError.into()),
    };
}

/// # Extract
/// A helper function that extracts data from a ByteBuffer. It uses the byteorder
/// crate to read the correct size words from the bytes of the actual raster.
fn extract<T: 'static + Copy + FromPrimitive>(bytes: Vec<u8>) -> Result<Vec<T>> {
    // wrap the bytes in a cursor
    let mut cursor = Cursor::new(bytes);
    // construct an output location
    let mut output: Vec<T> = vec![];
    // read values from cursor
    if TypeId::of::<T>() == TypeId::of::<u8>() {
        while let Ok(value) = cursor.read_u8() {
            if let Some(v) = T::from_u8(value) {
                output.push(v);
            }
        }
    } else if TypeId::of::<T>() == TypeId::of::<u16>() {
        while let Ok(value) = cursor.read_u16::<LittleEndian>() {
            if let Some(v) = T::from_u16(value) {
                output.push(v);
            }
        }
    } else if TypeId::of::<T>() == TypeId::of::<i16>() {
        while let Ok(value) = cursor.read_i16::<LittleEndian>() {
            if let Some(v) = T::from_i16(value) {
                output.push(v);
            }
        }
    } else if TypeId::of::<T>() == TypeId::of::<u32>() {
        while let Ok(value) = cursor.read_u32::<LittleEndian>() {
            if let Some(v) = T::from_u32(value) {
                output.push(v);
            }
        }
    } else if TypeId::of::<T>() == TypeId::of::<i32>() {
        while let Ok(value) = cursor.read_i32::<LittleEndian>() {
            if let Some(v) = T::from_i32(value) {
                output.push(v);
            }
        }
    } else if TypeId::of::<T>() == TypeId::of::<f32>() {
        while let Ok(value) = cursor.read_f32::<LittleEndian>() {
            if let Some(v) = T::from_f32(value) {
                output.push(v);
            }
        }
    } else if TypeId::of::<T>() == TypeId::of::<f64>() {
        while let Ok(value) = cursor.read_f64::<LittleEndian>() {
            if let Some(v) = T::from_f64(value) {
                output.push(v);
            }
        }
    } else {
        return Err(ConversionError.into());
    }
    // return the output
    Ok(output)
}
