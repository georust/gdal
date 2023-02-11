use crate::dataset::Dataset;
use crate::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::raster::{GdalDataType, GdalType};
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string};
use gdal_sys::{
    self, CPLErr, GDALColorEntry, GDALColorInterp, GDALColorTableH, GDALComputeRasterMinMax,
    GDALCreateColorRamp, GDALCreateColorTable, GDALDestroyColorTable, GDALGetPaletteInterpretation,
    GDALGetRasterStatistics, GDALMajorObjectH, GDALPaletteInterp, GDALRIOResampleAlg, GDALRWFlag,
    GDALRasterBandH, GDALRasterIOExtraArg, GDALSetColorEntry, GDALSetRasterColorTable,
};
use libc::c_int;
use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[cfg(feature = "ndarray")]
use ndarray::Array2;

use crate::errors::*;

/// Resampling algorithms used throughout various GDAL raster I/O operations.
///
/// # Example
///
/// ```rust
/// use gdal::Dataset;
/// # fn main() -> gdal::errors::Result<()> {
/// use gdal::raster::ResampleAlg;
/// let ds = Dataset::open("fixtures/tinymarble.tif")?;
/// let band1 = ds.rasterband(1)?;
/// let stats = band1.get_statistics(true, false)?.unwrap();
/// // Down-sample a image using cubic-spline interpolation
/// let buf = band1.read_as::<f64>((0, 0), ds.raster_size(), (2, 2), Some(ResampleAlg::CubicSpline))?;
/// // In this particular image, resulting data should be close to the overall average.
/// assert!(buf.data.iter().all(|c| (c - stats.mean).abs() < stats.std_dev / 2.0));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum ResampleAlg {
    /// Nearest neighbour
    NearestNeighbour = GDALRIOResampleAlg::GRIORA_NearestNeighbour,
    /// Bilinear (2x2 kernel)
    Bilinear = GDALRIOResampleAlg::GRIORA_Bilinear,
    /// Cubic Convolution Approximation (4x4 kernel)
    Cubic = GDALRIOResampleAlg::GRIORA_Cubic,
    /// Cubic B-Spline Approximation (4x4 kernel)
    CubicSpline = GDALRIOResampleAlg::GRIORA_CubicSpline,
    /// Lanczos windowed sinc interpolation (6x6 kernel)
    Lanczos = GDALRIOResampleAlg::GRIORA_Lanczos,
    /// Average
    Average = GDALRIOResampleAlg::GRIORA_Average,
    /// Mode (selects the value which appears most often of all the sampled points)
    Mode = GDALRIOResampleAlg::GRIORA_Mode,
    /// Gauss blurring
    Gauss = GDALRIOResampleAlg::GRIORA_Gauss,
}

impl ResampleAlg {
    /// Convert Rust enum discriminant to value expected by [`GDALRasterIOExtraArg`].
    pub fn to_gdal(&self) -> GDALRIOResampleAlg::Type {
        *self as GDALRIOResampleAlg::Type
    }
}

/// Wrapper type for gdal mask flags.
/// From the GDAL docs:
/// - `GMF_ALL_VALID`(0x01): There are no invalid pixels, all mask values will be 255. When used this will normally be the only flag set.
/// - `GMF_PER_DATASET`(0x02): The mask band is shared between all bands on the dataset.
/// - `GMF_ALPHA`(0x04): The mask band is actually an alpha band and may have values other than 0 and 255.
/// - `GMF_NODATA`(0x08): Indicates the mask is actually being generated from nodata values. (mutually exclusive of `GMF_ALPHA`)
pub struct GdalMaskFlags(i32);

impl GdalMaskFlags {
    const GMF_ALL_VALID: i32 = 0x01;
    const GMF_PER_DATASET: i32 = 0x02;
    const GMF_ALPHA: i32 = 0x04;
    const GMF_NODATA: i32 = 0x08;

    pub fn is_all_valid(&self) -> bool {
        self.0 & Self::GMF_ALL_VALID != 0
    }

    pub fn is_per_dataset(&self) -> bool {
        self.0 & Self::GMF_PER_DATASET != 0
    }

    pub fn is_alpha(&self) -> bool {
        self.0 & Self::GMF_ALPHA != 0
    }

    pub fn is_nodata(&self) -> bool {
        self.0 & Self::GMF_NODATA != 0
    }
}

/// Extra options used to read a raster.
///
/// For documentation, see `gdal_sys::GDALRasterIOExtraArg`.
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct RasterIOExtraArg {
    pub n_version: usize,
    pub e_resample_alg: ResampleAlg,
    pub pfn_progress: gdal_sys::GDALProgressFunc,
    p_progress_data: *mut libc::c_void,
    pub b_floating_point_window_validity: usize,
    pub df_x_off: f64,
    pub df_y_off: f64,
    pub df_x_size: f64,
    pub df_y_size: f64,
}

impl Default for RasterIOExtraArg {
    fn default() -> Self {
        Self {
            n_version: 1,
            pfn_progress: None,
            p_progress_data: std::ptr::null_mut(),
            e_resample_alg: ResampleAlg::NearestNeighbour,
            b_floating_point_window_validity: 0,
            df_x_off: 0.0,
            df_y_off: 0.0,
            df_x_size: 0.0,
            df_y_size: 0.0,
        }
    }
}

impl From<RasterIOExtraArg> for GDALRasterIOExtraArg {
    fn from(arg: RasterIOExtraArg) -> Self {
        let RasterIOExtraArg {
            n_version,
            e_resample_alg,
            pfn_progress,
            p_progress_data,
            b_floating_point_window_validity,
            df_x_off,
            df_y_off,
            df_x_size,
            df_y_size,
        } = arg;

        GDALRasterIOExtraArg {
            nVersion: n_version as c_int,
            eResampleAlg: e_resample_alg.to_gdal(),
            pfnProgress: pfn_progress,
            pProgressData: p_progress_data,
            bFloatingPointWindowValidity: b_floating_point_window_validity as c_int,
            dfXOff: df_x_off,
            dfYOff: df_y_off,
            dfXSize: df_x_size,
            dfYSize: df_y_size,
        }
    }
}

/// Represents a single band of a dataset.
///
/// This object carries the lifetime of the dataset that
/// contains it. This is necessary to prevent the dataset
/// from being dropped before the band.
pub struct RasterBand<'a> {
    c_rasterband: GDALRasterBandH,
    dataset: &'a Dataset,
}

impl<'a> RasterBand<'a> {
    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_rasterband(&self) -> GDALRasterBandH {
        self.c_rasterband
    }

    /// Create a RasterBand from a wrapped C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_rasterband(dataset: &'a Dataset, c_rasterband: GDALRasterBandH) -> Self {
        RasterBand {
            c_rasterband,
            dataset,
        }
    }

    /// The size of a preferred I/O raster block size as a (cols, rows) tuple. Reading/writing
    /// chunks corresponding to the returned value should offer the best performance.
    pub fn block_size(&self) -> (usize, usize) {
        let mut size_x = 0;
        let mut size_y = 0;

        unsafe { gdal_sys::GDALGetBlockSize(self.c_rasterband, &mut size_x, &mut size_y) };
        (size_x as usize, size_y as usize)
    }

    /// Get x-size (width, or number of column) of the band.
    /// *Note*: This may not be the same as number of columns of the owning [`Dataset`], due to scale.
    pub fn x_size(&self) -> usize {
        let out;
        unsafe {
            out = gdal_sys::GDALGetRasterBandXSize(self.c_rasterband);
        }
        out as usize
    }

    /// Get y-size (height, or number of rows) of the band
    /// *Note*: This may not be the same as number of rows of the owning [`Dataset`], due to scale.
    pub fn y_size(&self) -> usize {
        let out;
        unsafe { out = gdal_sys::GDALGetRasterBandYSize(self.c_rasterband) }
        out as usize
    }

    /// Get dimensions of the band, as a (cols, rows) tuple.
    /// *Note*: This may not be the same as `raster_size` on the `owning_dataset` due to scale.
    ///
    pub fn size(&self) -> (usize, usize) {
        (self.x_size(), self.y_size())
    }

    /// Read data from this band into a slice, where `T` implements [`GdalType`]
    ///
    /// # Arguments
    /// * `window` - the window position from top left
    /// * `window_size` - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * `size` - the desired size to read
    /// * `buffer` - a slice to hold the data (length must equal product of size parameter)
    /// * `e_resample_alg` - the resample algorithm used for the interpolation. Default: `NearestNeighbor`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use gdal::Dataset;
    /// use gdal::raster::{GdalDataType, ResampleAlg};
    /// let dataset = Dataset::open("fixtures/m_3607824_se_17_1_20160620_sub.tif")?;
    /// let band1 = dataset.rasterband(1)?;
    /// assert_eq!(band1.band_type(), GdalDataType::UInt8);
    /// let size = 4;
    /// let mut buf = vec![0; size*size];
    /// band1.read_into_slice::<u8>((0, 0), band1.size(), (size, size), buf.as_mut_slice(), Some(ResampleAlg::Bilinear))?;
    /// assert_eq!(buf, [101u8, 119, 94, 87, 92, 110, 92, 87, 91, 90, 89, 87, 92, 91, 88, 88]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_into_slice<T: Copy + GdalType>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        buffer: &mut [T],
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<()> {
        let pixels = size.0 * size.1;
        assert_eq!(buffer.len(), pixels);

        let resample_alg = e_resample_alg.unwrap_or(ResampleAlg::NearestNeighbour);

        let mut options: GDALRasterIOExtraArg = RasterIOExtraArg {
            e_resample_alg: resample_alg,
            ..Default::default()
        }
        .into();

        let options_ptr: *mut GDALRasterIOExtraArg = &mut options;

        let rv = unsafe {
            gdal_sys::GDALRasterIOEx(
                self.c_rasterband,
                GDALRWFlag::GF_Read,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                buffer.as_mut_ptr() as GDALRasterBandH,
                size.0 as c_int,
                size.1 as c_int,
                T::gdal_ordinal(),
                0,
                0,
                options_ptr,
            )
        };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }

        Ok(())
    }

    /// Read a [`Buffer<T>`] from this band, where `T` implements [`GdalType`].
    ///
    /// # Arguments
    /// * `window` - the window position from top left
    /// * `window_size` - the window size (GDAL will interpolate data if `window_size` != `buffer_size`)
    /// * `buffer_size` - the desired size of the 'Buffer'
    /// * `e_resample_alg` - the resample algorithm used for the interpolation. Default: `NearestNeighbor`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use gdal::Dataset;
    /// use gdal::raster::{GdalDataType, ResampleAlg};
    /// let dataset = Dataset::open("fixtures/m_3607824_se_17_1_20160620_sub.tif")?;
    /// let band1 = dataset.rasterband(1)?;
    /// assert_eq!(band1.band_type(), GdalDataType::UInt8);
    /// let size = 4;
    /// let buf = band1.read_as::<u8>((0, 0), band1.size(), (size, size), Some(ResampleAlg::Bilinear))?;
    /// assert_eq!(buf.size, (size, size));
    /// assert_eq!(buf.data, [101u8, 119, 94, 87, 92, 110, 92, 87, 91, 90, 89, 87, 92, 91, 88, 88]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_as<T: Copy + GdalType>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<Buffer<T>> {
        let pixels = size.0 * size.1;
        let mut data: Vec<T> = Vec::with_capacity(pixels);

        let resample_alg = e_resample_alg.unwrap_or(ResampleAlg::NearestNeighbour);

        let mut options: GDALRasterIOExtraArg = RasterIOExtraArg {
            e_resample_alg: resample_alg,
            ..Default::default()
        }
        .into();

        let options_ptr: *mut GDALRasterIOExtraArg = &mut options;

        // Safety: the GDALRasterIOEx writes
        // exactly pixel elements into the slice, before we
        // read from this slice. This paradigm is suggested
        // in the rust std docs
        // (https://doc.rust-lang.org/std/vec/struct.Vec.html#examples-18)
        let rv = unsafe {
            gdal_sys::GDALRasterIOEx(
                self.c_rasterband,
                GDALRWFlag::GF_Read,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                data.as_mut_ptr() as GDALRasterBandH,
                size.0 as c_int,
                size.1 as c_int,
                T::gdal_ordinal(),
                0,
                0,
                options_ptr,
            )
        };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }

        unsafe {
            data.set_len(pixels);
        };

        Ok(Buffer { size, data })
    }

    #[cfg(feature = "ndarray")]
    #[cfg_attr(docsrs, doc(cfg(feature = "array")))]
    /// Read a [`Array2<T>`] from this band, where `T` implements [`GdalType`].
    ///
    /// # Arguments
    /// * `window` - the window position from top left
    /// * `window_size` - the window size (GDAL will interpolate data if window_size != array_size)
    /// * `array_size` - the desired size of the 'Array'
    /// * `e_resample_alg` - the resample algorithm used for the interpolation
    ///
    /// # Note
    /// The Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis).
    pub fn read_as_array<T: Copy + GdalType>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        array_size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<Array2<T>> {
        let data = self.read_as::<T>(window, window_size, array_size, e_resample_alg)?;

        // Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis)
        Ok(Array2::from_shape_vec(
            (array_size.1, array_size.0),
            data.data,
        )?)
    }

    /// Read the full band as a [`Buffer<T>`], where `T` implements [`GdalType`].
    pub fn read_band_as<T: Copy + GdalType>(&self) -> Result<Buffer<T>> {
        let size = self.size();
        self.read_as::<T>((0, 0), (size.0, size.1), (size.0, size.1), None)
    }

    #[cfg(feature = "ndarray")]
    #[cfg_attr(docsrs, doc(cfg(feature = "array")))]
    /// Read a [`Array2<T>`] from a [`Dataset`] block, where `T` implements [`GdalType`].
    ///
    /// # Arguments
    /// * `block_index` - the block index
    ///
    /// # Notes
    /// The Matrix shape is (rows, cols) and raster shape is (cols in x-axis, rows in y-axis).
    pub fn read_block<T: Copy + GdalType>(&self, block_index: (usize, usize)) -> Result<Array2<T>> {
        let size = self.block_size();
        let pixels = size.0 * size.1;
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
            return Err(_last_cpl_err(rv));
        }

        unsafe {
            data.set_len(pixels);
        };

        Array2::from_shape_vec((size.1, size.0), data).map_err(Into::into)
    }

    /// Write a [`Buffer<T>`] into a [`Dataset`].
    ///
    /// # Arguments
    /// * `window` - the window position from top left
    /// * `window_size` - the window size (GDAL will interpolate data if window_size != Buffer.size)
    /// * `buffer` - the data to write into the window
    ///
    pub fn write<T: GdalType + Copy>(
        &mut self,
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
                T::gdal_ordinal(),
                0,
                0,
            )
        };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    /// Returns the pixel datatype of this band.
    pub fn band_type(&self) -> GdalDataType {
        let ordinal = unsafe { gdal_sys::GDALGetRasterDataType(self.c_rasterband) };
        ordinal.try_into().unwrap_or(GdalDataType::Unknown)
    }

    /// Returns the no-data value of this band.
    pub fn no_data_value(&self) -> Option<f64> {
        let mut pb_success = 1;
        let no_data =
            unsafe { gdal_sys::GDALGetRasterNoDataValue(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(no_data);
        }
        None
    }

    /// Set the no data value of this band.
    ///
    /// If `no_data` is `None`, any existing no-data value is deleted.
    pub fn set_no_data_value(&mut self, no_data: Option<f64>) -> Result<()> {
        let rv = if let Some(no_data) = no_data {
            unsafe { gdal_sys::GDALSetRasterNoDataValue(self.c_rasterband, no_data) }
        } else {
            unsafe { gdal_sys::GDALDeleteRasterNoDataValue(self.c_rasterband) }
        };

        if rv != CPLErr::CE_None {
            Err(_last_cpl_err(rv))
        } else {
            Ok(())
        }
    }

    /// Returns the color interpretation of this band.
    pub fn color_interpretation(&self) -> ColorInterpretation {
        let interp_index = unsafe { gdal_sys::GDALGetRasterColorInterpretation(self.c_rasterband) };
        ColorInterpretation::from_c_int(interp_index).unwrap()
    }

    /// Set the color interpretation for this band.
    pub fn set_color_interpretation(&mut self, interp: ColorInterpretation) -> Result<()> {
        let interp_index = interp.c_int();
        let rv =
            unsafe { gdal_sys::GDALSetRasterColorInterpretation(self.c_rasterband, interp_index) };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    /// Get the color table for this band if it has one.
    pub fn color_table(&self) -> Option<ColorTable> {
        let c_color_table = unsafe { gdal_sys::GDALGetRasterColorTable(self.c_rasterband) };
        if c_color_table.is_null() {
            return None;
        }
        Some(ColorTable::from_c_color_table(c_color_table))
    }

    /// Set the color table for this band.
    ///
    /// See [`ColorTable`] for usage example.
    pub fn set_color_table(&mut self, colors: &ColorTable) {
        unsafe { GDALSetRasterColorTable(self.c_rasterband, colors.c_color_table) };
    }

    /// Returns the scale of this band if set.
    pub fn scale(&self) -> Option<f64> {
        let mut pb_success = 1;
        let scale = unsafe { gdal_sys::GDALGetRasterScale(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(scale);
        }
        None
    }

    /// Set the scale for this band.
    pub fn set_scale(&mut self, scale: f64) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALSetRasterScale(self.c_rasterband, scale) };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    /// Returns the offset of this band if set.
    pub fn offset(&self) -> Option<f64> {
        let mut pb_success = 1;
        let offset = unsafe { gdal_sys::GDALGetRasterOffset(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(offset);
        }
        None
    }

    /// Set the offset for this band.
    pub fn set_offset(&mut self, offset: f64) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALSetRasterOffset(self.c_rasterband, offset) };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
    }

    /// Get actual block size (at the edges) when block size
    /// does not divide band size.
    #[cfg(any(all(major_is_2, minor_ge_2), major_ge_3))] // GDAL 2.2 .. 2.x or >= 3
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
            return Err(_last_cpl_err(rv));
        }
        Ok((block_size_x as usize, block_size_y as usize))
    }

    pub fn overview_count(&self) -> Result<i32> {
        unsafe { Ok(gdal_sys::GDALGetOverviewCount(self.c_rasterband)) }
    }

    pub fn overview(&self, overview_index: isize) -> Result<RasterBand<'a>> {
        unsafe {
            let c_band = self.c_rasterband;
            let overview = gdal_sys::GDALGetOverview(c_band, overview_index as libc::c_int);
            if overview.is_null() {
                return Err(_last_null_pointer_err("GDALGetOverview"));
            }
            Ok(RasterBand::from_c_rasterband(self.dataset, overview))
        }
    }

    /// Return the unit of the rasterband.
    /// If there is no unit, the empty string is returned.
    pub fn unit(&self) -> String {
        let str_ptr = unsafe { gdal_sys::GDALGetRasterUnitType(self.c_rasterband) };

        if str_ptr.is_null() {
            return String::new();
        }

        _string(str_ptr)
    }

    /// Read the band mask flags for a GDAL `RasterBand`.
    pub fn mask_flags(&self) -> Result<GdalMaskFlags> {
        let band_mask_flags = unsafe { gdal_sys::GDALGetMaskFlags(self.c_rasterband) };

        Ok(GdalMaskFlags(band_mask_flags))
    }

    /// Create a new mask band for the layer.
    /// `shared_between_all_bands` indicates if all bands of the dataset use the same mask.
    pub fn create_mask_band(&mut self, shared_between_all_bands: bool) -> Result<()> {
        let flags = if shared_between_all_bands {
            GdalMaskFlags::GMF_PER_DATASET // It is the only valid flag here.
        } else {
            0x00
        };

        let rv = unsafe { gdal_sys::GDALCreateMaskBand(self.c_rasterband, flags) };
        if rv != 0 {
            return Err(_last_cpl_err(rv));
        };
        Ok(())
    }

    /// Open the mask-`Rasterband`
    pub fn open_mask_band(&self) -> Result<RasterBand> {
        unsafe {
            let mask_band_ptr = gdal_sys::GDALGetMaskBand(self.c_rasterband);
            if mask_band_ptr.is_null() {
                return Err(_last_null_pointer_err("GDALGetMaskBand"));
            }
            let mask_band = RasterBand::from_c_rasterband(self.dataset, mask_band_ptr);
            Ok(mask_band)
        }
    }

    /// Fetch image statistics.
    ///
    /// Returns the minimum, maximum, mean and standard deviation of all pixel values in this band.
    /// If approximate statistics are sufficient, the `is_approx_ok` flag can be set to true in which case overviews, or a subset of image tiles may be used in computing the statistics.
    ///
    /// If `force` is `false` results will only be returned if it can be done quickly (i.e. without scanning the data).
    /// If force` is `false` and results cannot be returned efficiently, the method will return `None`.
    ///
    /// Note that file formats using PAM (Persistent Auxiliary Metadata) services will generally cache statistics in the .pam file allowing fast fetch after the first request.
    ///
    /// This methods is a wrapper for [`GDALGetRasterStatistics`](https://gdal.org/api/gdalrasterband_cpp.html#_CPPv4N14GDALRasterBand13GetStatisticsEiiPdPdPdPd).
    ///
    pub fn get_statistics(&self, force: bool, is_approx_ok: bool) -> Result<Option<StatisticsAll>> {
        let mut statistics = StatisticsAll {
            min: 0.,
            max: 0.,
            mean: 0.,
            std_dev: 0.,
        };

        let rv = unsafe {
            GDALGetRasterStatistics(
                self.c_rasterband,
                libc::c_int::from(is_approx_ok),
                libc::c_int::from(force),
                &mut statistics.min,
                &mut statistics.max,
                &mut statistics.mean,
                &mut statistics.std_dev,
            )
        };

        match CplErrType::from(rv) {
            CplErrType::None => Ok(Some(statistics)),
            CplErrType::Warning => Ok(None),
            _ => Err(_last_cpl_err(rv)),
        }
    }

    /// Compute the min/max values for a band.
    ///
    /// If `is_approx_ok` is `true`, then the band’s GetMinimum()/GetMaximum() will be trusted.
    /// If it doesn’t work, a subsample of blocks will be read to get an approximate min/max.
    /// If the band has a nodata value it will be excluded from the minimum and maximum.
    ///
    /// If `is_approx_ok` is `false`, then all pixels will be read and used to compute an exact range.
    ///
    /// This methods is a wrapper for [`GDALComputeRasterMinMax`](https://gdal.org/api/gdalrasterband_cpp.html#_CPPv4N14GDALRasterBand19ComputeRasterMinMaxEiPd).
    ///
    pub fn compute_raster_min_max(&self, is_approx_ok: bool) -> Result<StatisticsMinMax> {
        let mut min_max = [0., 0.];

        // TODO: The C++ method actually returns a CPLErr, but the C interface does not expose it.
        unsafe {
            GDALComputeRasterMinMax(
                self.c_rasterband,
                libc::c_int::from(is_approx_ok),
                &mut min_max as *mut f64,
            )
        };

        Ok(StatisticsMinMax {
            min: min_max[0],
            max: min_max[1],
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct StatisticsMinMax {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, PartialEq)]
pub struct StatisticsAll {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
}

impl<'a> MajorObject for RasterBand<'a> {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH {
        self.c_rasterband
    }
}

impl<'a> Metadata for RasterBand<'a> {}

pub struct Buffer<T: GdalType> {
    pub size: (usize, usize),
    pub data: Vec<T>,
}

impl<T: GdalType> Buffer<T> {
    pub fn new(size: (usize, usize), data: Vec<T>) -> Buffer<T> {
        Buffer { size, data }
    }
}

pub type ByteBuffer = Buffer<u8>;

/// Represents a color interpretation of a RasterBand
#[derive(Debug, PartialEq, Eq)]
pub enum ColorInterpretation {
    /// Undefined
    Undefined,
    /// Grayscale
    GrayIndex,
    /// Paletted (see associated color table)
    PaletteIndex,
    /// Red band of RGBA image
    RedBand,
    /// Green band of RGBA image
    GreenBand,
    /// Blue band of RGBA image
    BlueBand,
    /// Alpha (0=transparent, 255=opaque)
    AlphaBand,
    /// Hue band of HLS image
    HueBand,
    /// Saturation band of HLS image
    SaturationBand,
    /// Lightness band of HLS image
    LightnessBand,
    /// Cyan band of CMYK image
    CyanBand,
    /// Magenta band of CMYK image
    MagentaBand,
    /// Yellow band of CMYK image
    YellowBand,
    /// Black band of CMYK image
    BlackBand,
    /// Y Luminance
    YCbCrSpaceYBand,
    /// Cb Chroma
    YCbCrSpaceCbBand,
    /// Cr Chroma
    YCbCrSpaceCrBand,
}

impl ColorInterpretation {
    /// Creates a color interpretation from its C API int value.
    pub fn from_c_int(color_interpretation: GDALColorInterp::Type) -> Option<Self> {
        match color_interpretation {
            GDALColorInterp::GCI_Undefined => Some(Self::Undefined),
            GDALColorInterp::GCI_GrayIndex => Some(Self::GrayIndex),
            GDALColorInterp::GCI_PaletteIndex => Some(Self::PaletteIndex),
            GDALColorInterp::GCI_RedBand => Some(Self::RedBand),
            GDALColorInterp::GCI_GreenBand => Some(Self::GreenBand),
            GDALColorInterp::GCI_BlueBand => Some(Self::BlueBand),
            GDALColorInterp::GCI_AlphaBand => Some(Self::AlphaBand),
            GDALColorInterp::GCI_HueBand => Some(Self::HueBand),
            GDALColorInterp::GCI_SaturationBand => Some(Self::SaturationBand),
            GDALColorInterp::GCI_LightnessBand => Some(Self::LightnessBand),
            GDALColorInterp::GCI_CyanBand => Some(Self::CyanBand),
            GDALColorInterp::GCI_MagentaBand => Some(Self::MagentaBand),
            GDALColorInterp::GCI_YellowBand => Some(Self::YellowBand),
            GDALColorInterp::GCI_BlackBand => Some(Self::BlackBand),
            GDALColorInterp::GCI_YCbCr_YBand => Some(Self::YCbCrSpaceYBand),
            GDALColorInterp::GCI_YCbCr_CbBand => Some(Self::YCbCrSpaceCbBand),
            GDALColorInterp::GCI_YCbCr_CrBand => Some(Self::YCbCrSpaceCrBand),
            _ => None,
        }
    }

    /// Returns the C API int value of this color interpretation.
    pub fn c_int(&self) -> GDALColorInterp::Type {
        match self {
            Self::Undefined => GDALColorInterp::GCI_Undefined,
            Self::GrayIndex => GDALColorInterp::GCI_GrayIndex,
            Self::PaletteIndex => GDALColorInterp::GCI_PaletteIndex,
            Self::RedBand => GDALColorInterp::GCI_RedBand,
            Self::GreenBand => GDALColorInterp::GCI_GreenBand,
            Self::BlueBand => GDALColorInterp::GCI_BlueBand,
            Self::AlphaBand => GDALColorInterp::GCI_AlphaBand,
            Self::HueBand => GDALColorInterp::GCI_HueBand,
            Self::SaturationBand => GDALColorInterp::GCI_SaturationBand,
            Self::LightnessBand => GDALColorInterp::GCI_LightnessBand,
            Self::CyanBand => GDALColorInterp::GCI_CyanBand,
            Self::MagentaBand => GDALColorInterp::GCI_MagentaBand,
            Self::YellowBand => GDALColorInterp::GCI_YellowBand,
            Self::BlackBand => GDALColorInterp::GCI_BlackBand,
            Self::YCbCrSpaceYBand => GDALColorInterp::GCI_YCbCr_YBand,
            Self::YCbCrSpaceCbBand => GDALColorInterp::GCI_YCbCr_CbBand,
            Self::YCbCrSpaceCrBand => GDALColorInterp::GCI_YCbCr_CrBand,
        }
    }

    /// Creates a color interpretation from its name.
    pub fn from_name(name: &str) -> Result<Self> {
        let c_str_interp_name = CString::new(name)?;
        let interp_index =
            unsafe { gdal_sys::GDALGetColorInterpretationByName(c_str_interp_name.as_ptr()) };
        Ok(Self::from_c_int(interp_index).unwrap())
    }

    /// Returns the name of this color interpretation.
    pub fn name(&self) -> String {
        let rv = unsafe { gdal_sys::GDALGetColorInterpretationName(self.c_int()) };
        _string(rv)
    }
}

/// Types of color interpretations for a [`ColorTable`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PaletteInterpretation {
    /// Grayscale
    Gray,
    /// Red, Green, Blue and Alpha
    Rgba,
    /// Cyan, Magenta, Yellow and Black
    Cmyk,
    /// Hue, Lightness and Saturation
    Hls,
}

impl PaletteInterpretation {
    /// Creates a Rust [`PaletteInterpretation`] from  a C API [`GDALPaletteInterp`] value.
    fn from_c_int(palette_interpretation: GDALPaletteInterp::Type) -> Self {
        match palette_interpretation {
            GDALPaletteInterp::GPI_Gray => Self::Gray,
            GDALPaletteInterp::GPI_RGB => Self::Rgba,
            GDALPaletteInterp::GPI_CMYK => Self::Cmyk,
            GDALPaletteInterp::GPI_HLS => Self::Hls,
            _ => unreachable!("GDAL has implemented a new type of `GDALPaletteInterp`"),
        }
    }

    /// Returns the C API int value of this palette interpretation.
    pub fn c_int(&self) -> GDALPaletteInterp::Type {
        match self {
            Self::Gray => GDALPaletteInterp::GPI_Gray,
            Self::Rgba => GDALPaletteInterp::GPI_RGB,
            Self::Cmyk => GDALPaletteInterp::GPI_CMYK,
            Self::Hls => GDALPaletteInterp::GPI_HLS,
        }
    }
}

/// Grayscale [`ColorTable`] entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GrayEntry {
    pub g: i16,
}

/// Red, green, blue, alpha [`ColorTable`] entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RgbaEntry {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

/// Cyan, magenta, yellow, black [`ColorTable`] entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CmykEntry {
    pub c: i16,
    pub m: i16,
    pub y: i16,
    pub k: i16,
}

/// Hue, lightness, saturation [`ColorTable`] entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HlsEntry {
    pub h: i16,
    pub l: i16,
    pub s: i16,
}

/// Options for defining [`ColorInterpretation::PaletteIndex`] entries in a [`ColorTable`].
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ColorEntry {
    Gray(GrayEntry),
    Rgba(RgbaEntry),
    Cmyk(CmykEntry),
    Hls(HlsEntry),
}

impl ColorEntry {
    /// Instantiate a greyscale color entry
    pub fn grey(g: i16) -> Self {
        Self::Gray(GrayEntry { g })
    }

    /// Instantiate an red, green, blue, alpha color entry
    pub fn rgba(r: i16, g: i16, b: i16, a: i16) -> Self {
        Self::Rgba(RgbaEntry { r, g, b, a })
    }

    /// Instantiate a cyan, magenta, yellow, black color entry
    pub fn cmyk(c: i16, m: i16, y: i16, k: i16) -> Self {
        Self::Cmyk(CmykEntry { c, m, y, k })
    }

    /// Instantiate a hue, lightness, saturation color entry
    pub fn hls(h: i16, l: i16, s: i16) -> Self {
        Self::Hls(HlsEntry { h, l, s })
    }

    /// Get the ['PaletteInterpretation'] describing `self`.
    pub fn palette_interpretation(&self) -> PaletteInterpretation {
        match self {
            ColorEntry::Gray(_) => PaletteInterpretation::Gray,
            ColorEntry::Rgba(_) => PaletteInterpretation::Rgba,
            ColorEntry::Cmyk(_) => PaletteInterpretation::Cmyk,
            ColorEntry::Hls(_) => PaletteInterpretation::Hls,
        }
    }

    /// Create from a C [`GDALColorEntry`].
    fn from(e: GDALColorEntry, interp: PaletteInterpretation) -> ColorEntry {
        match interp {
            PaletteInterpretation::Gray => ColorEntry::Gray(GrayEntry { g: e.c1 }),
            PaletteInterpretation::Rgba => ColorEntry::Rgba(RgbaEntry {
                r: e.c1,
                g: e.c2,
                b: e.c3,
                a: e.c4,
            }),
            PaletteInterpretation::Cmyk => ColorEntry::Cmyk(CmykEntry {
                c: e.c1,
                m: e.c2,
                y: e.c3,
                k: e.c4,
            }),
            PaletteInterpretation::Hls => ColorEntry::Hls(HlsEntry {
                h: e.c1,
                l: e.c2,
                s: e.c3,
            }),
        }
    }
}

impl From<&ColorEntry> for GDALColorEntry {
    fn from(e: &ColorEntry) -> Self {
        match e {
            ColorEntry::Gray(e) => GDALColorEntry {
                c1: e.g,
                c2: 0,
                c3: 0,
                c4: 0,
            },
            ColorEntry::Rgba(e) => GDALColorEntry {
                c1: e.r,
                c2: e.g,
                c3: e.b,
                c4: e.a,
            },
            ColorEntry::Cmyk(e) => GDALColorEntry {
                c1: e.c,
                c2: e.m,
                c3: e.y,
                c4: e.k,
            },
            ColorEntry::Hls(e) => GDALColorEntry {
                c1: e.h,
                c2: e.l,
                c3: e.s,
                c4: 0,
            },
        }
    }
}

// For more compact debug output, skip enum wrapper.
impl Debug for ColorEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorEntry::Gray(e) => e.fmt(f),
            ColorEntry::Rgba(e) => e.fmt(f),
            ColorEntry::Cmyk(e) => e.fmt(f),
            ColorEntry::Hls(e) => e.fmt(f),
        }
    }
}

/// Color table for raster bands that use [`ColorInterpretation::PaletteIndex`] color interpretation.
///
///
/// # Example
///
/// ```rust, no_run
/// use gdal::{Dataset, DriverManager};
/// use gdal::raster::{ColorEntry, ColorTable, PaletteInterpretation};
/// # fn main() -> gdal::errors::Result<()> {
///
/// // Open source multinomial classification raster
/// let ds = Dataset::open("fixtures/labels.tif")?;
///
/// // Create in-memory copy to mutate
/// let mem_driver = DriverManager::get_driver_by_name("MEM")?;
/// let ds = ds.create_copy(&mem_driver, "<mem>", &[])?;
/// let mut band = ds.rasterband(1)?;
/// assert!(band.color_table().is_none());
///
/// // Create a new color table for 3 classes + no-data
/// let mut ct = ColorTable::new(PaletteInterpretation::Rgba);
/// ct.set_color_entry(0, &ColorEntry::rgba(255, 255, 0, 255));
/// ct.set_color_entry(1, &ColorEntry::rgba(0, 255, 255, 255));
/// ct.set_color_entry(2, &ColorEntry::rgba(255, 0, 255, 255));
/// ct.set_color_entry(255, &ColorEntry::rgba(0, 0, 0, 0));
/// band.set_color_table(&ct);
///
/// // Render a PNG
/// let png_driver = DriverManager::get_driver_by_name("PNG")?;
/// ds.create_copy(&png_driver, "/tmp/labels.png", &[])?;
///
/// # Ok(())
/// # }
/// ```
pub struct ColorTable<'a> {
    palette_interpretation: PaletteInterpretation,
    c_color_table: GDALColorTableH,
    /// If `true`, Rust is responsible for deallocating color table pointed to by
    /// `c_color_table`, which is the case when instantiated directly, as opposed to
    /// when read via [`RasterBand::color_table`].
    rust_owned: bool,
    phantom_raster_band: PhantomData<&'a RasterBand<'a>>,
}

impl<'a> ColorTable<'a> {
    /// Instantiate a new color table with the given palette interpretation.
    pub fn new(interp: PaletteInterpretation) -> Self {
        let c_color_table = unsafe { GDALCreateColorTable(interp.c_int()) };
        Self {
            palette_interpretation: interp,
            c_color_table,
            rust_owned: true,
            phantom_raster_band: PhantomData,
        }
    }

    /// Constructs a color ramp from one color entry to another.
    ///
    /// `start_index` and `end_index` must be `0..=255`.
    ///
    /// Returns `None` if `start_color` and `end_color` do not have the same [`PaletteInterpretation`].
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::{ColorEntry, ColorTable};
    /// # fn main() -> gdal::errors::Result<()> {
    /// // Create a 16 step blue to white color table.
    /// let ct = ColorTable::color_ramp(
    ///     0, &ColorEntry::rgba(0, 0, 255, 255),
    ///     15, &ColorEntry::rgba(255, 255, 255, 255)
    /// )?;
    /// println!("{ct:?}");
    /// # Ok(())
    /// # }
    pub fn color_ramp(
        start_index: u8,
        start_color: &ColorEntry,
        end_index: u8,
        end_color: &ColorEntry,
    ) -> Result<ColorTable<'a>> {
        if start_color.palette_interpretation() != end_color.palette_interpretation() {
            Err(GdalError::BadArgument(
                "start_color and end_color must have the same palette_interpretation".into(),
            ))
        } else {
            let ct = ColorTable::new(start_color.palette_interpretation());
            unsafe {
                GDALCreateColorRamp(
                    ct.c_color_table,
                    start_index as c_int,
                    &start_color.into(),
                    end_index as c_int,
                    &end_color.into(),
                );
            }
            Ok(ct)
        }
    }

    /// Wrap C color table
    fn from_c_color_table(c_color_table: GDALColorTableH) -> Self {
        let interp_index = unsafe { GDALGetPaletteInterpretation(c_color_table) };
        ColorTable {
            palette_interpretation: PaletteInterpretation::from_c_int(interp_index),
            c_color_table,
            rust_owned: false,
            phantom_raster_band: PhantomData,
        }
    }

    /// How the values of this color table are interpreted.
    pub fn palette_interpretation(&self) -> PaletteInterpretation {
        self.palette_interpretation
    }

    /// Get the number of color entries in this color table.
    pub fn entry_count(&self) -> usize {
        unsafe { gdal_sys::GDALGetColorEntryCount(self.c_color_table) as usize }
    }

    /// Get a color entry.
    pub fn entry(&self, index: usize) -> Option<ColorEntry> {
        let color_entry = unsafe {
            let c_color_entry = gdal_sys::GDALGetColorEntry(self.c_color_table, index as i32);
            if c_color_entry.is_null() {
                return None;
            }
            *c_color_entry
        };
        Some(ColorEntry::from(color_entry, self.palette_interpretation))
    }

    /// Get a color entry as RGB.
    ///
    /// Returns `None` if `palette_interpretation` is not `Rgba`.
    pub fn entry_as_rgb(&self, index: usize) -> Option<RgbaEntry> {
        let mut color_entry = GDALColorEntry {
            c1: 0,
            c2: 0,
            c3: 0,
            c4: 0,
        };
        if unsafe {
            gdal_sys::GDALGetColorEntryAsRGB(self.c_color_table, index as i32, &mut color_entry)
        } == 0
        {
            return None;
        }
        Some(RgbaEntry {
            r: color_entry.c1,
            g: color_entry.c2,
            b: color_entry.c3,
            a: color_entry.c4,
        })
    }

    /// Set entry in the RasterBand color table.
    ///
    /// The `entry` variant type must match `palette_interpretation`.
    ///
    /// The table is grown as needed to hold the supplied index, filling in gaps with
    /// the default [`GDALColorEntry`] value.
    pub fn set_color_entry(&mut self, index: u16, entry: &ColorEntry) {
        unsafe { GDALSetColorEntry(self.c_color_table, index as c_int, &entry.into()) }
    }
}

impl Drop for ColorTable<'_> {
    fn drop(&mut self) {
        if self.rust_owned {
            unsafe { GDALDestroyColorTable(self.c_color_table) }
        }
    }
}

impl Default for ColorTable<'_> {
    fn default() -> Self {
        Self::new(PaletteInterpretation::Rgba)
    }
}

impl Debug for ColorTable<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let entries = (0..self.entry_count())
            .filter_map(|i| self.entry(i))
            .collect::<Vec<_>>();

        f.debug_struct("ColorTable")
            .field("palette_interpretation", &self.palette_interpretation)
            .field("entries", &entries)
            .finish()
    }
}
