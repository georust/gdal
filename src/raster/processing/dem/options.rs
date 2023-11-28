use std::num::NonZeroUsize;
use std::ptr;
use std::ptr::NonNull;

use gdal_sys::{
    GDALDEMProcessingOptions, GDALDEMProcessingOptionsFree, GDALDEMProcessingOptionsNew,
};

use crate::cpl::CslStringList;
use crate::errors;
use crate::utils::_last_null_pointer_err;

/// Payload for [`GDALDEMProcessing`]. Intended for internal use only.
pub struct GdalDEMProcessingOptions(NonNull<GDALDEMProcessingOptions>);

impl GdalDEMProcessingOptions {
    pub fn new(opts: &CslStringList) -> errors::Result<Self> {
        // GDAL copies the relevant value out of `opts`, we don't need to keep them alive:
        // https://github.com/OSGeo/gdal/blob/59eaaed3168f49e8a7a3821730277aff68a86d16/apps/gdaldem_lib.cpp#L3770
        let popts = unsafe { GDALDEMProcessingOptionsNew(opts.as_ptr(), ptr::null_mut()) };
        match NonNull::new(popts) {
            Some(popts) => Ok(Self(popts)),
            None => Err(_last_null_pointer_err("GDALDEMProcessingOptionsNew")),
        }
    }

    pub fn as_ptr(&self) -> *const GDALDEMProcessingOptions {
        self.0.as_ptr()
    }
}

impl Drop for GdalDEMProcessingOptions {
    fn drop(&mut self) {
        unsafe { GDALDEMProcessingOptionsFree(self.0.as_ptr()) };
    }
}

/// DEM processor mode, to stringify and pass to [`gdal_sys::GDALDEMProcessing`].
#[derive(Debug, Clone, Copy)]
pub enum DemAlg {
    /// Computes the azimuth that the slopes in some DEM data are facing.
    Aspect,
    /// Uses a configuration file to colorized a DEM dataset.
    ColorRelief,
    /// Performs hill-shade rendering of DEM data.
    Hillshade,
    /// Computes the roughness from DEM data, which is the largest difference between the central pixel and its surrounding cells.
    Roughness,
    /// Computes slope values from DEM data.
    Slope,
    /// Computes the Topographic Position Index from DEM data, which is the difference between the central pixel and the mean of its surrounding cells.
    Tpi,
    /// Computes the Topographic Roughness Index from DEM data, which measures the difference between the central pixel and the surrounding cells.
    Tri,
}

impl DemAlg {
    pub(super) fn to_gdal_option(self) -> &'static str {
        match self {
            DemAlg::Aspect => "aspect",
            DemAlg::ColorRelief => "color-relief",
            DemAlg::Hillshade => "hillshade",
            DemAlg::Roughness => "roughness",
            DemAlg::Slope => "slope",
            DemAlg::Tpi => "TPI",
            DemAlg::Tri => "TRI",
        }
    }
}

/// Slope and slope-related (aspect, hillshade) processing algorithms.
///
/// The literature suggests `ZevenbergenThorne` to be more suited to smooth landscapes,
/// whereas `Horn` performs better on rougher terrain.
#[derive(Debug, Clone, Copy)]
pub enum DemSlopeAlg {
    /// The Horn's formula, which performs better on rougher terrain.
    Horn,
    /// Zevenbergen & Thorne, which works better on smooth terrain.
    ZevenbergenThorne,
}

impl DemSlopeAlg {
    pub(super) fn to_gdal_option(self) -> &'static str {
        match self {
            DemSlopeAlg::Horn => "Horn",
            DemSlopeAlg::ZevenbergenThorne => "ZevenbergenThorne",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CommonOptions {
    pub(crate) input_band: Option<NonZeroUsize>,
    pub(crate) compute_edges: Option<bool>,
    pub(crate) output_format: Option<String>,
    pub(crate) additional_options: CslStringList,
}

macro_rules! common_dem_options {
    () => {
        /// Specify which band in the input [`Dataset`][crate::Dataset] to read from.
        ///
        /// Defaults to the first band.
        pub fn with_input_band(&mut self, band: NonZeroUsize) -> &mut Self {
            self.common_options.input_band = Some(band);
            self
        }

        /// Explicitly specify output raster format.
        ///
        /// This is equivalent to the `-of <format>` CLI flag accepted by many GDAL tools.
        ///
        /// The value of `format` must be the identifier of a driver supported by the runtime
        /// environment's GDAL library (e.g. `COG`, `JPEG`, `VRT`, etc.). A list of these identifiers
        /// is available from `gdalinfo --formats`:
        ///
        /// ```text
        /// ❯ gdalinfo --formats
        /// Supported Formats:
        ///   VRT -raster,multidimensional raster- (rw+v): Virtual Raster
        ///   DERIVED -raster- (ro): Derived datasets using VRT pixel functions
        ///   GTiff -raster- (rw+vs): GeoTIFF
        ///   COG -raster- (wv): Cloud optimized GeoTIFF generator
        ///   NITF -raster- (rw+vs): National Imagery Transmission Format
        /// ...
        /// ```
        ///
        pub fn with_output_format(&mut self, format: &str) -> &mut Self {
            self.common_options.output_format = Some(format.to_owned());
            self
        }

        /// Compute values at image edges.
        ///
        /// If true, causes interpolation of values at image edges or if a no-data value is found
        /// in the 3x3 processing window.
        pub fn with_compute_edges(&mut self, state: bool) -> &mut Self {
            self.common_options.compute_edges = Some(state);
            self
        }

        /// Additional generic options to be included.
        pub fn with_additional_options(&mut self, extra_options: CslStringList) -> &mut Self {
            self.common_options
                .additional_options
                .extend(&extra_options);
            self
        }

        /// Private utility to convert common options into [`CslStringList`] options.
        fn store_common_options_to(&self, opts: &mut CslStringList) -> errors::Result<()> {
            if matches!(self.common_options.compute_edges, Some(true)) {
                opts.add_string("-compute_edges")?;
            }

            if let Some(band) = self.common_options.input_band {
                opts.add_string("-b")?;
                opts.add_string(&band.to_string())?;
            }

            if let Some(of) = &self.common_options.output_format {
                opts.add_string("-of")?;
                opts.add_string(of)?;
            }

            if !self.common_options.additional_options.is_empty() {
                opts.extend(&self.common_options.additional_options);
            }

            Ok(())
        }
    };
}

pub(crate) use common_dem_options;
