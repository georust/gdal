use crate::dataset::Dataset;
use crate::gdal_major_object::MajorObject;
use crate::metadata::Metadata;
use crate::raster::{GDALDataType, GdalType};
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string};
use gdal_sys::{
    self, CPLErr, GDALColorInterp, GDALMajorObjectH, GDALRWFlag, GDALRasterBandH,
    GDALRasterIOExtraArg,
};
use libc::c_int;
use std::ffi::CString;

#[cfg(feature = "ndarray")]
use ndarray::Array2;

use crate::errors::*;

/// Resampling algorithms, map GDAL defines
#[derive(Debug, Copy, Clone)]
pub enum ResampleAlg {
    /// Nearest neighbour
    NearestNeighbour,
    /// Bilinear (2x2 kernel)
    Bilinear,
    /// Cubic Convolution Approximation (4x4 kernel)
    Cubic,
    /// Cubic B-Spline Approximation (4x4 kernel)
    CubicSpline,
    /// Lanczos windowed sinc interpolation (6x6 kernel)
    Lanczos,
    /// Average
    Average,
    /// Mode (selects the value which appears most often of all the sampled points)
    Mode,
    /// Gauss blurring
    Gauss,
}

fn map_resample_alg(alg: &ResampleAlg) -> u32 {
    match alg {
        ResampleAlg::NearestNeighbour => 0,
        ResampleAlg::Bilinear => 1,
        ResampleAlg::Cubic => 2,
        ResampleAlg::CubicSpline => 3,
        ResampleAlg::Lanczos => 4,
        ResampleAlg::Average => 5,
        ResampleAlg::Mode => 6,
        ResampleAlg::Gauss => 7,
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
            eResampleAlg: map_resample_alg(&e_resample_alg),
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

    /// Read data from this band into a slice. T implements 'GdalType'
    ///
    /// # Arguments
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * size - the desired size to read
    /// * buffer - a slice to hold the data (length must equal product of size parameter)
    /// * e_resample_alg - the resample algorithm used for the interpolation
    pub fn read_into_slice<T: Copy + GdalType>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        buffer: &mut [T],
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<()> {
        let pixels = (size.0 * size.1) as usize;
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
                T::gdal_type(),
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

    /// Read a 'Buffer<T>' from this band. T implements 'GdalType'
    ///
    /// # Arguments
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * buffer_size - the desired size of the 'Buffer'
    /// * e_resample_alg - the resample algorithm used for the interpolation
    pub fn read_as<T: Copy + GdalType>(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<Buffer<T>> {
        let pixels = (size.0 * size.1) as usize;

        let mut data: Vec<T> = Vec::with_capacity(pixels);

        // Safety: the read_into_slice line below writes
        // exactly pixel elements into the slice, before we
        // read from this slice. This paradigm is suggested
        // in the rust std docs
        // (https://doc.rust-lang.org/std/vec/struct.Vec.html#examples-18)
        unsafe {
            data.set_len(pixels);
        };
        self.read_into_slice(window, window_size, size, &mut data, e_resample_alg)?;

        Ok(Buffer { size, data })
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

    /// Read the full band as a 'Buffer<T>'.
    pub fn read_band_as<T: Copy + GdalType>(&self) -> Result<Buffer<T>> {
        let size = self.size();
        self.read_as::<T>(
            (0, 0),
            (size.0 as usize, size.1 as usize),
            (size.0 as usize, size.1 as usize),
            None,
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
            return Err(_last_cpl_err(rv));
        }

        unsafe {
            data.set_len(pixels);
        };

        Array2::from_shape_vec((size.1, size.0), data).map_err(Into::into)
    }

    /// Write a 'Buffer<T>' into a 'Dataset'.
    /// # Arguments
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != Buffer.size)
    /// * buffer - the data to write into the window
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
                T::gdal_type(),
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
    pub fn band_type(&self) -> GDALDataType::Type {
        unsafe { gdal_sys::GDALGetRasterDataType(self.c_rasterband) }
    }

    /// Returns the no data value of this band.
    pub fn no_data_value(&self) -> Option<f64> {
        let mut pb_success = 1;
        let no_data =
            unsafe { gdal_sys::GDALGetRasterNoDataValue(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(no_data as f64);
        }
        None
    }

    /// Set the no data value of this band.
    pub fn set_no_data_value(&mut self, no_data: f64) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALSetRasterNoDataValue(self.c_rasterband, no_data) };
        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }
        Ok(())
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

    /// Returns the scale of this band if set.
    pub fn scale(&self) -> Option<f64> {
        let mut pb_success = 1;
        let scale = unsafe { gdal_sys::GDALGetRasterScale(self.c_rasterband, &mut pb_success) };
        if pb_success == 1 {
            return Some(scale as f64);
        }
        None
    }

    /// Returns the offset of this band if set.
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
#[derive(Debug, PartialEq)]
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
