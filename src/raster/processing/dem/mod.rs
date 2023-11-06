//! Digital Elevation Model (DEM) processing routines.
//!
//! This module provides bindings to the algorithms in the
//! [`gdaldem` tool](https://gdal.org/programs/gdaldem.html#gdaldem).
//!
//! The routines assume an open dataset containing customary digital elevation model data.
//! This includes assumptions that `x` (east-west), `y` (north-south), and `z` (elevation) units are identical.
//! If `x` and `y` units are identical, but `z` (elevation) units are different,
//! `hillshade` and `slope` support a scale setting to set the ratio of vertical units to horizontal.
//! See [`SlopeOptions::with_scale`] for details.
//!
//! # Examples
//!
//! Examples may be found associated with the following methods:
//!
//! * [`DemProcessing::aspect`]
//! * [`DemProcessing::color_relief`]
//! * [`DemProcessing::hillshade`]
//! * [`DemProcessing::roughness`]
//! * [`DemProcessing::slope`]
//! * [`DemProcessing::terrain_ruggedness_index`]
//! * [`DemProcessing::topographic_position_index`]
//!

mod aspect;
mod color_relief;
mod hillshade;
mod roughness;
mod slope;
mod tpi;
mod tri;

pub use aspect::*;
pub use color_relief::*;
pub use hillshade::*;
pub use slope::*;
pub use tri::*;

use crate::cpl::CslStringList;
use crate::errors::Result;
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _path_to_c_string};
use crate::Dataset;
use gdal_sys::{
    CPLErr, GDALDEMProcessing, GDALDEMProcessingOptions, GDALDEMProcessingOptionsFree,
    GDALDEMProcessingOptionsNew,
};
use libc::c_int;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::ptr;
use std::ptr::NonNull;

/// Digital Elevation Model (DEM) processing routines.
pub trait DemProcessing {
    /// Slope aspect-angle routine for DEM datasets.
    ///
    /// This method outputs a 32-bit float raster with values between 0° and 360°
    /// representing the azimuth that slopes are facing. The definition of the azimuth is such that:
    ///
    /// * 0° means that the slope is facing the North,
    /// * 90° it's facing the East,
    /// * 180° it's facing the South and;
    /// * 270° it's facing the West (provided that the top of your input raster is north oriented).
    ///
    /// By default, the aspect value `-9999` is used as the no-data value to indicate undefined aspect in flat
    /// areas with slope=0. See [`AspectOptions::with_zero_for_flat`] for alternative.
    ///
    /// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::Dataset;
    /// # fn main() -> gdal::errors::Result<()> {
    /// use std::path::Path;
    /// use gdal::raster::processing::dem::{DemProcessing, AspectOptions, DemSlopeAlg};
    /// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
    /// let mut opts = AspectOptions::new();
    /// opts
    ///     .with_algorithm(DemSlopeAlg::Horn)
    ///     .with_zero_for_flat(true);
    /// let aspect_ds = ds.aspect(Path::new("target/dem-hills-aspect.tiff"), &opts)?;
    /// let stats = aspect_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
    /// println!("{stats:#?}");
    /// # Ok(())
    /// # }
    /// ```
    /// The resulting output is:
    ///
    /// ```text
    /// StatisticsAll {
    ///     min: 0.0,
    ///     max: 359.9951171875,
    ///     mean: 165.72752499997543,
    ///     std_dev: 98.5901999514453,
    /// }
    /// ```
    ///
    /// See: [`gdaldem aspect`](https://gdal.org/programs/gdaldem.html#aspect) for details,
    fn aspect<P: AsRef<Path>>(&self, dest_file: P, options: &AspectOptions) -> Result<Dataset>;

    /// Generate a color-relief rendering of DEM data.
    ///
    /// This routine outputs a 3-band (RGB) or 4-band (RGBA) raster with values computed from
    /// the elevation and a text-based color configuration file.
    ///
    /// The color configuration file contains associations between various elevation values
    /// and the corresponding desired color. See [`ColorReliefOptions::new`] for details.
    ///
    /// By default, the colors between the given elevation
    /// values are blended smoothly and the result is a nice colorized DEM.
    /// The [`ColorMatchingMode::ExactColorEntry`] or [`ColorMatchingMode::NearestColorEntry`] options
    /// can be used to avoid that linear interpolation for values that don't match an index of
    /// the color configuration file. See [`ColorMatchingMode`] for details.
    ///
    /// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::Dataset;
    /// # fn main() -> gdal::errors::Result<()> {
    /// use std::path::Path;
    /// use gdal::raster::processing::dem::{ColorReliefOptions, DemProcessing};
    /// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
    /// let mut opts = ColorReliefOptions::new("fixtures/color-relief.clr");
    /// opts.with_alpha(true);
    /// let hs_ds = ds.color_relief(Path::new("target/dem-hills-relief.tiff"), &opts)?;
    /// // Note: Output will actually be a 4-band raster.
    /// let stats = hs_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
    /// println!("{stats:#?}");
    /// # Ok(())
    /// # }
    /// ```
    /// The resulting output is:
    ///
    /// ```text
    /// StatisticsAll {
    ///     min: 50.0,
    ///     max: 255.0,
    ///     mean: 206.85964128690114,
    ///     std_dev: 52.73836661993173,
    /// }
    /// ```
    /// See: [`gdaldem color-relief`](https://gdal.org/programs/gdaldem.html#color-relief) for details,
    ///
    fn color_relief<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &ColorReliefOptions,
    ) -> Result<Dataset>;

    /// Performs hill-shade rendering of DEM data.
    ///
    /// This routine outputs an 8-bit raster with a nice shaded relief effect.
    /// It’s very useful for visualizing the terrain.
    /// You can optionally specify the azimuth and altitude of the light source,
    /// a vertical exaggeration factor and a scaling factor to account for
    /// differences between vertical and horizontal units.
    ///
    /// The value `0` is used as the output no-data value.
    ///
    /// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::Dataset;
    /// # fn main() -> gdal::errors::Result<()> {
    /// use std::path::Path;
    /// use gdal::raster::processing::dem::{DemProcessing, DemSlopeAlg, HillshadeOptions, ShadingMode};
    /// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
    /// let mut opts = HillshadeOptions::new();
    /// opts
    ///     .with_algorithm(DemSlopeAlg::Horn)
    ///     .with_z_factor(4.0)
    ///     .with_scale(98473.0)
    ///     .with_shading_mode(ShadingMode::Combined);
    /// let hs_ds = ds.hillshade(Path::new("target/dem-hills-shade.tiff"), &opts)?;
    /// let stats = hs_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
    /// println!("{stats:#?}");
    /// # Ok(())
    /// # }
    /// ```
    /// The resulting output is:
    ///
    /// ```text
    /// StatisticsAll {
    ///     min: 31.0,
    ///     max: 255.0,
    ///     mean: 234.71988886841396,
    ///     std_dev: 30.556285572761446,
    /// }
    /// ```
    /// See: [`gdaldem hillshade`](https://gdal.org/programs/gdaldem.html#hillshade) for details,
    ///
    fn hillshade<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &HillshadeOptions,
    ) -> Result<Dataset>;

    /// Roughness routine for DEM datasets.
    ///
    /// This processor outputs a single-band raster with values computed from the elevation.
    /// Roughness is the largest inter-cell difference of a central pixel and its surrounding cell,
    /// as defined in Wilson et al (2007, Marine Geodesy 30:3-35).
    ///
    /// The value `-9999` is used as the output no-data value.
    ///
    /// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use gdal::Dataset;
    /// use gdal::raster::processing::dem::DemProcessing;
    /// use std::path::Path;
    /// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
    /// let roughness_ds = ds.roughness(Path::new("target/dem-hills-roughness.tiff"), &Default::default())?;
    /// let stats = roughness_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
    /// println!("{stats:#?}");
    /// # Ok(())
    /// # }
    /// ```
    /// The resulting output is:
    ///
    /// ```text
    /// StatisticsAll {
    ///     min: 0.0,
    ///     max: 14.361007690429688,
    ///     mean: 1.5128357817365072,
    ///     std_dev: 2.0120679959607686,
    /// }
    /// ```
    ///
    /// See: [`gdaldem roughness`](https://gdal.org/programs/gdaldem.html#roughness) for details.
    fn roughness<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &DemCommonOptions,
    ) -> Result<Dataset>;

    /// Slope computation routine for DEM datasets.
    ///
    /// This method will take a DEM raster Dataset and output a 32-bit float raster with slope values.
    ///
    /// You have the option of specifying the type of slope value you want:
    /// [degrees or percent slope](SlopeOptions::with_percentage_results).
    ///
    /// In cases where the horizontal units differ from the vertical units, you can also supply
    /// a [scaling factor](SlopeOptions::with_scale).
    ///
    /// The value `-9999` is used as the output no-data value.
    ///
    /// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use std::path::Path;
    /// use gdal::Dataset;
    /// use gdal::raster::processing::dem::{DemProcessing, DemSlopeAlg, SlopeOptions};
    /// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
    /// let mut opts = SlopeOptions::new();
    /// opts
    ///     .with_algorithm(DemSlopeAlg::Horn)
    ///     .with_percentage_results(true)
    ///     .with_scale(98473.0);
    /// let slope_ds = ds.slope(Path::new("target/dem-hills-slope.tiff"), &opts)?;
    /// let stats = slope_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
    /// println!("{stats:#?}");
    /// # Ok(())
    /// # }
    /// ```
    /// The resulting output is:
    ///
    /// ```text
    /// StatisticsAll {
    ///     min: 0.0,
    ///     max: 65.44061279296875,
    ///     mean: 6.171115265248098,
    ///     std_dev: 8.735612161193623,
    /// }
    /// ```
    ///
    /// See: [`gdaldem slope`](https://gdal.org/programs/gdaldem.html#slope) for details,
    fn slope<P: AsRef<Path>>(&self, dest_file: P, options: &SlopeOptions) -> Result<Dataset>;

    /// Topographic Position Index (TPI) routine for DEM datasets
    ///
    /// This method outputs a single-band raster with values computed from the elevation.
    /// A Topographic Position Index is defined as the difference between a central pixel and the
    /// mean of its surrounding cells (see Wilson et al 2007, Marine Geodesy 30:3-35).
    ///
    /// The value `-9999` is used as the output no-data value.
    ///
    /// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use std::path::Path;
    /// use gdal::Dataset;
    /// use gdal::raster::processing::dem::DemProcessing;
    /// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
    /// let tpi_ds = ds.topographic_position_index(Path::new("target/dem-hills-tpi.tiff"), &Default::default())?;
    /// let stats = tpi_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
    /// println!("{stats:#?}");
    /// # Ok(())
    /// # }
    /// ```
    /// The resulting output is:
    ///
    /// ```text
    /// StatisticsAll {
    ///     min: -4.7376708984375,
    ///     max: 4.7724151611328125,
    ///     mean: 0.00012131847966825689,
    ///     std_dev: 0.48943078832474257,
    /// }
    /// ```
    ///    
    /// See: [`gdaldem tpi`](https://gdal.org/programs/gdaldem.html#tpi) for details.
    fn topographic_position_index<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &DemCommonOptions,
    ) -> Result<Dataset>;

    /// Terrain Ruggedness Index (TRI) routine for DEM datasets
    ///
    /// This method outputs a single-band raster with values computed from the elevation.
    /// TRI stands for Terrain Ruggedness Index, which measures the difference between a
    /// central pixel and its surrounding cells.
    ///
    /// The value `-9999` is used as the output no-data value.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// # fn main() -> gdal::errors::Result<()> {
    /// use std::path::Path;
    /// use gdal::Dataset;
    /// use gdal::raster::processing::dem::{DemProcessing, DemTriAlg, TriOptions};
    /// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
    /// let mut opts = TriOptions::new();
    /// opts.with_algorithm(DemTriAlg::Wilson);
    /// let tri_ds = ds.terrain_ruggedness_index(Path::new("target/dem-hills-tri.tiff"), &opts)?;
    /// let stats = tri_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
    /// println!("{stats:#?}");
    /// # Ok(())
    /// # }
    /// ```
    /// The resulting output is:
    ///
    /// ```text
    /// StatisticsAll {
    ///     min: 0.0,
    ///     max: 4.983623504638672,
    ///     mean: 0.49063101456532326,
    ///     std_dev: 0.6719356336694824,
    /// }
    /// ```
    ///
    /// See: [`gdaldem tri`](https://gdal.org/programs/gdaldem.html#tri) for details.
    fn terrain_ruggedness_index<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &TriOptions,
    ) -> Result<Dataset>;
}

/// Implementation of Digital Elevation Model (DEM) processing routines for [`Dataset`].
impl DemProcessing for Dataset {
    fn aspect<P: AsRef<Path>>(&self, dest_file: P, options: &AspectOptions) -> Result<Self> {
        dem_eval(
            self,
            dest_file.as_ref(),
            DemAlg::Aspect,
            &options.to_options_list(),
            None,
        )
    }

    fn color_relief<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &ColorReliefOptions,
    ) -> Result<Dataset> {
        let colors = options.color_config();
        dem_eval(
            self,
            dest_file.as_ref(),
            DemAlg::ColorRelief,
            &options.to_options_list(),
            Some(colors),
        )
    }

    fn hillshade<P: AsRef<Path>>(&self, dest_file: P, options: &HillshadeOptions) -> Result<Self> {
        dem_eval(
            self,
            dest_file.as_ref(),
            DemAlg::Hillshade,
            &options.to_options_list(),
            None,
        )
    }

    fn roughness<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &DemCommonOptions,
    ) -> Result<Dataset> {
        dem_eval(
            self,
            dest_file.as_ref(),
            DemAlg::Roughness,
            &options.to_options_list(),
            None,
        )
    }

    fn slope<P: AsRef<Path>>(&self, dest_file: P, options: &SlopeOptions) -> Result<Dataset> {
        dem_eval(
            self,
            dest_file.as_ref(),
            DemAlg::Slope,
            &options.to_options_list(),
            None,
        )
    }

    fn topographic_position_index<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &DemCommonOptions,
    ) -> Result<Dataset> {
        dem_eval(
            self,
            dest_file.as_ref(),
            DemAlg::Tpi,
            &options.to_options_list(),
            None,
        )
    }

    fn terrain_ruggedness_index<P: AsRef<Path>>(
        &self,
        dest_file: P,
        options: &TriOptions,
    ) -> Result<Self> {
        dem_eval(
            self,
            dest_file.as_ref(),
            DemAlg::Tri,
            &options.to_options_list(),
            None,
        )
    }
}

/// Options common across all DEM operations.
///
/// # Example
///
/// ```rust
/// use gdal::raster::processing::dem::{DemCommonOptions, DemCommonOptionsOwner};
/// let mut opts = DemCommonOptions::new();
/// opts.with_compute_edges(true)
///     .with_output_format("GTiff")
///     .with_additional_options("CPL_DEBUG=TRUE".parse().unwrap());
/// ```
#[derive(Debug, Clone, Default)]
pub struct DemCommonOptions {
    input_band: Option<NonZeroUsize>,
    compute_edges: Option<bool>,
    output_format: Option<String>,
    additional_options: Option<CslStringList>,
}

impl DemCommonOptions {
    pub fn new() -> Self {
        Default::default()
    }

    /// Render relevant common options into [`CslStringList`] values, as compatible with
    /// [`GDALDEMProcessing`].
    fn to_options_list(&self) -> CslStringList {
        let mut opts = CslStringList::new();

        if self.compute_edges == Some(true) {
            opts.add_string("-compute_edges").unwrap();
        }

        if let Some(band) = self.input_band {
            opts.add_string("-b").unwrap();
            opts.add_string(&band.to_string()).unwrap();
        }

        if let Some(of) = &self.output_format {
            opts.add_string("-of").unwrap();
            opts.add_string(of).unwrap();
        }

        if let Some(extra) = &self.additional_options {
            opts.extend(extra);
        }

        opts
    }
}

/// Trait for DEM routine options wrapping [`DemCommonOptions`].
pub trait DemCommonOptionsOwner {
    /// Fetch a reference to the owned [`DemCommonOptions`].
    fn opts(&self) -> &DemCommonOptions;
    /// Fetch a mutable reference to the owned [`DemCommonOptions`].
    fn opts_mut(&mut self) -> &mut DemCommonOptions;

    /// Specify which band in the input [`Dataset`] to read from.
    ///
    /// Defaults to the first band.
    fn with_input_band(&mut self, band: NonZeroUsize) -> &mut Self {
        self.opts_mut().input_band = Some(band);
        self
    }

    /// Fetch the specified input band to read from.
    fn input_band(&self) -> Option<NonZeroUsize> {
        self.opts().input_band
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
    fn with_output_format(&mut self, format: &str) -> &mut Self {
        self.opts_mut().output_format = Some(format.to_owned());
        self
    }

    /// Fetch the specified output format driver identifier.
    fn output_format(&self) -> Option<String> {
        self.opts().output_format.clone()
    }

    /// Compute values at image edges.
    ///
    /// This causes interpolation of values at image edges or if a no-data value is found
    /// in the 3x3 processing window.
    fn with_compute_edges(&mut self, state: bool) -> &mut Self {
        self.opts_mut().compute_edges = Some(state);
        self
    }

    /// Fetch the specified edge computation value.
    ///
    /// Returns `None` if not specified.
    fn compute_edges(&self) -> Option<bool> {
        self.opts().compute_edges
    }

    /// Additional generic options to be included.
    fn with_additional_options(&mut self, extra_options: CslStringList) -> &mut Self {
        self.opts_mut().additional_options = Some(extra_options);
        self
    }

    /// Fetch any specified additional options.
    fn additional_options(&self) -> Option<&CslStringList> {
        self.opts().additional_options.as_ref()
    }
}

/// Exposes common DEM routine options when routine doesn't have any additional options.
impl DemCommonOptionsOwner for DemCommonOptions {
    fn opts(&self) -> &DemCommonOptions {
        self
    }

    fn opts_mut(&mut self) -> &mut DemCommonOptions {
        self
    }
}

/// Execute the processor on the given [`Dataset`].
fn dem_eval(
    src: &Dataset,
    dst_file: &Path,
    alg: DemAlg,
    options: &CslStringList,
    color_relief_config: Option<PathBuf>,
) -> Result<Dataset> {
    let popts = GdalDEMProcessingOptions::new(options)?;
    let mode = CString::new(alg.to_string())?;
    let dest = _path_to_c_string(dst_file)?;
    let cfile = color_relief_config.and_then(|p| _path_to_c_string(&p).ok());
    let cfile_ptr = cfile.as_deref().map(CStr::as_ptr).unwrap_or(ptr::null());

    let mut pb_usage_error: c_int = 0;
    let out_ds = unsafe {
        // Docs: https://github.com/OSGeo/gdal/blob/6a3584b2fea51f92022d24ad8036749ba1b98958/apps/gdaldem_lib.cpp#L3184
        GDALDEMProcessing(
            dest.as_ptr(),
            src.c_dataset(),
            mode.as_ptr(),
            cfile_ptr,
            popts.as_ptr(),
            &mut pb_usage_error as *mut c_int,
        )
    };

    if pb_usage_error != 0 {
        Err(_last_cpl_err(CPLErr::CE_Failure))
    } else {
        let out_ds = unsafe { Dataset::from_c_dataset(out_ds) };
        Ok(out_ds)
    }
}

/// DEM processor mode, to stringify and pass to [`GDALDEMProcessing`].
#[derive(Debug, Clone, Copy)]
enum DemAlg {
    Aspect,
    ColorRelief,
    Hillshade,
    Roughness,
    Slope,
    Tpi,
    Tri,
}

impl Display for DemAlg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ColorRelief => f.write_str("color-relief"),
            _ => {
                let s = format!("{self:?}").to_lowercase();
                f.write_str(&s)
            }
        }
    }
}

/// Slope and slope-related (aspect, hillshade) processing algorithms.
///
/// The literature suggests `ZevenbergenThorne` to be more suited to smooth landscapes,
/// whereas `Horn` performs better on rougher terrain.
#[derive(Debug, Clone, Copy)]
pub enum DemSlopeAlg {
    Horn,
    ZevenbergenThorne,
}

impl Display for DemSlopeAlg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

/// Payload for [`GDALDEMProcessing`]. Intended for internal use only.
struct GdalDEMProcessingOptions<'opts>(
    NonNull<GDALDEMProcessingOptions>,
    PhantomData<&'opts CslStringList>,
);

impl<'opts> GdalDEMProcessingOptions<'opts> {
    fn new(opts: &'opts CslStringList) -> Result<Self> {
        let popts = unsafe { GDALDEMProcessingOptionsNew(opts.as_ptr(), ptr::null_mut()) };
        if popts.is_null() {
            return Err(_last_null_pointer_err("GDALDEMProcessingOptionsNew"));
        }
        Ok(Self(unsafe { NonNull::new_unchecked(popts) }, PhantomData))
    }

    fn as_ptr(&self) -> *const GDALDEMProcessingOptions {
        self.0.as_ptr()
    }
}

impl Drop for GdalDEMProcessingOptions<'_> {
    fn drop(&mut self) {
        unsafe { GDALDEMProcessingOptionsFree(self.0.as_ptr()) };
    }
}
