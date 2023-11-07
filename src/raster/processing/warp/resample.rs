use crate::errors::{GdalError, Result};
use gdal_sys::GDALResampleAlg;

/// GDAL Warp Resampling Algorithm
#[derive(Debug, Copy, Clone, Default)]
#[repr(u32)]
pub enum WarpResampleAlg {
    /// Nearest neighbour (select on one input pixel)
    #[default]
    NearestNeighbour = GDALResampleAlg::GRA_NearestNeighbour,
    /// Bilinear (2x2 kernel)
    Bilinear = GDALResampleAlg::GRA_Bilinear,
    /// Cubic Convolution Approximation (4x4 kernel)
    Cubic = GDALResampleAlg::GRA_Cubic,
    /// Cubic B-Spline Approximation (4x4 kernel)
    CubicSpline = GDALResampleAlg::GRA_CubicSpline,
    /// Lanczos windowed sinc interpolation (6x6 kernel)
    Lanczos = GDALResampleAlg::GRA_Lanczos,
    /// Average (computes the weighted average of all non-NODATA contributing\npixels)
    Average = GDALResampleAlg::GRA_Average,
    /// Mode (selects the value which appears most often of all the sampled\npoints)
    Mode = GDALResampleAlg::GRA_Mode,
    /// Max (selects maximum of all non-NODATA contributing pixels)
    Max = GDALResampleAlg::GRA_Max,
    /// Min (selects minimum of all non-NODATA contributing pixels)
    Min = GDALResampleAlg::GRA_Min,
    /// Med (selects median of all non-NODATA contributing pixels)
    Med = GDALResampleAlg::GRA_Med,
    /// Q1 (selects first quartile of all non-NODATA contributing pixels)
    Q1 = GDALResampleAlg::GRA_Q1,
    /// Q3 (selects third quartile of all non-NODATA contributing pixels)
    Q3 = GDALResampleAlg::GRA_Q3,
    /// Sum (weighed sum of all non-NODATA contributing pixels). Added in\nGDAL 3.1
    Sum = GDALResampleAlg::GRA_Sum,
    /// RMS (weighted root mean square (quadratic mean) of all non-NODATA\ncontributing pixels)
    RMS = GDALResampleAlg::GRA_RMS,
}

impl WarpResampleAlg {
    pub fn to_gdal(&self) -> GDALResampleAlg::Type {
        *self as GDALResampleAlg::Type
    }
    pub fn from_gdal(alg: GDALResampleAlg::Type) -> Result<Self> {
        Ok(match alg {
            GDALResampleAlg::GRA_NearestNeighbour => WarpResampleAlg::NearestNeighbour,
            GDALResampleAlg::GRA_Bilinear => WarpResampleAlg::Bilinear,
            GDALResampleAlg::GRA_Cubic => WarpResampleAlg::Cubic,
            GDALResampleAlg::GRA_CubicSpline => WarpResampleAlg::CubicSpline,
            GDALResampleAlg::GRA_Lanczos => WarpResampleAlg::Lanczos,
            GDALResampleAlg::GRA_Average => WarpResampleAlg::Average,
            GDALResampleAlg::GRA_Mode => WarpResampleAlg::Mode,
            GDALResampleAlg::GRA_Max => WarpResampleAlg::Max,
            GDALResampleAlg::GRA_Min => WarpResampleAlg::Min,
            GDALResampleAlg::GRA_Med => WarpResampleAlg::Med,
            GDALResampleAlg::GRA_Q1 => WarpResampleAlg::Q1,
            GDALResampleAlg::GRA_Q3 => WarpResampleAlg::Q3,
            GDALResampleAlg::GRA_Sum => WarpResampleAlg::Sum,
            GDALResampleAlg::GRA_RMS => WarpResampleAlg::RMS,
            o => {
                return Err(GdalError::BadArgument(format!(
                    "Ordinal {o} does not map to a supported WarpResampleAlg"
                )))
            }
        })
    }
}
